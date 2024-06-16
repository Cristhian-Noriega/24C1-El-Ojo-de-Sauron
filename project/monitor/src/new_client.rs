use std::{
    collections::HashMap,
    io::{ErrorKind, Write},
    net::TcpStream,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use mqtt::model::{
    components::{
        encoded_string::EncodedString, qos::QoS, topic_level::TopicLevel, topic_name::TopicName,
    },
    packet::Packet,
    packets::{connect::Connect, puback::Puback, publish::Publish},
};

use crate::{
    channels_tasks::{MonitorAction, UIAction},
    drone::{self, Drone},
    incident::Incident,
    monitor::Monitor,
    ui_application::UIApplication,
};

pub fn client_run(address: String) -> Result<(), String> {
    // Create the channels to communicate between the monitor and the UI
    let (monitor_sender, monitor_receiver) = channel();
    let (ui_sender, ui_receiver) = channel();

    // Connect to the server
    let stream = match connect_to_server(address) {
        Ok(stream) => stream,
        Err(e) => {
            return Err(format!("Error connecting to server: {:?}", e));
        }
    };

    // monitor start in a thread to avoid blocking the main thread
    let monitor_thread = thread::spawn(move || {
        start_monitor(stream, monitor_sender, ui_receiver);
    });

    // start the ui in the main thread
    match start_ui(ui_sender, monitor_receiver) {
        Ok(_) => {}
        Err(_) => {
            return Err("Error starting UI".to_string());
        }
    }

    // wait for the monitor thread to finish
    monitor_thread.join().unwrap();

    Ok(())
}

fn connect_to_server(address: String) -> std::io::Result<TcpStream> {
    println!("\nConnecting to address: {:?}", address);
    let mut to_server_stream = TcpStream::connect(address)?;

    let client_id_bytes: Vec<u8> = b"monitor".to_vec();
    let client_id = EncodedString::new(client_id_bytes);
    let will = None;
    let login = None; // TODO: Add login
    let connect = Connect::new(false, 0, client_id, will, login);

    let _ = to_server_stream.write(connect.to_bytes().as_slice());

    match Packet::from_bytes(&mut to_server_stream) {
        Ok(Packet::Connack(connack)) => {
            println!(
                "Received Connack packet with return code: {:?} and sessionPresent: {:?}\n",
                connack.connect_return_code(),
                connack.session_present()
            );
            Ok(to_server_stream)
        }
        _ => Err(std::io::Error::new(ErrorKind::Other, "No connack received")),
    }
}

fn start_ui(
    ui_sender: Sender<UIAction>,
    from_monitor_receiver: Receiver<MonitorAction>,
) -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        ..Default::default()
    };

    eframe::run_native(
        "Monitor",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::new(UIApplication::new(
                cc.egui_ctx.clone(),
                ui_sender,
                from_monitor_receiver,
            ))
        }),
    )
}

const DRONE_DATA: &[u8] = b"drone-data";
const DRONE_REGISTER: &[u8] = b"drone-register";
const NEW_INCIDENT: &[u8] = b"new-incident";
const CLOSE_INCIDENT: &[u8] = b"close-incident";
const SEPARATOR: char = ';';
const CAMERA_DATA: &[u8] = b"camera-data";

fn start_monitor(
    stream: TcpStream,
    monitor_sender: Sender<MonitorAction>,
    ui_receiver: Receiver<UIAction>,
) {
    let monitor = Arc::new(Mutex::new(Monitor::new()));

    let stream = Arc::new(Mutex::new(stream));

    // Thread for receiving data
    let stream_clone = Arc::clone(&stream);
    let monitor_sender_clone = monitor_sender.clone();
    let monitor_clone = Arc::clone(&monitor);
    thread::spawn(move || loop {
        let mut stream = stream_clone.lock().unwrap();
        match Packet::from_bytes(&mut *stream) {
            Ok(Packet::Puback(_)) => {
                // Handle Puback if needed
            }
            Ok(Packet::Publish(publish)) => {
                handle_publish_packet(&monitor_clone, publish, &monitor_sender_clone);
            }
            Ok(_) => {}
           
            Err(e) => {
                println!("Error reading packet from server: {:?}", e);
                break;
            }
        }
    });

    // Thread for sending data
    let stream_clone = Arc::clone(&stream);
    thread::spawn(move || loop {
        match ui_receiver.recv() {
            Ok(action) => handle_ui_action(&stream_clone, &monitor, action),
            Err(e) => {
                println!("Error receiving UI action: {:?}", e);
                break;
            }
        }
    });

    // Main thread loop to keep the monitor running
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}

fn handle_publish_packet(
    monitor: &Arc<Mutex<Monitor>>,
    publish: Publish,
    monitor_sender: &Sender<MonitorAction>,
) {
    let topic_name = publish.topic();
    let topic_levels = topic_name.levels();

    println!("Received publish packet with topic: {:?}", topic_levels);

    if topic_levels.len() == 2 && topic_levels[0] == DRONE_DATA {
        println!("Identified that it is drone data");
        let id = topic_levels[1].as_slice();

        let id = String::from_utf8_lossy(id.to_vec().as_slice()).to_string();
        let content = publish.message();
        let content_str = std::str::from_utf8(&content).unwrap();
        let splitted_content: Vec<&str> = content_str.split(SEPARATOR).collect();

        let x_coordinate = splitted_content[0].parse::<f64>().unwrap();
        let y_coordinate = splitted_content[1].parse::<f64>().unwrap();
        let state = splitted_content[2].to_string();
        let battery = splitted_content[3].parse::<usize>().unwrap();

        let mut monitor = monitor.lock().unwrap();

        if !monitor.has_registered_drone(&id) {
            let drone = Drone::new(id.clone(), state, battery, x_coordinate, y_coordinate);
            monitor.add_drone(drone);
        } else {
            monitor.update_drone(&id, state, battery, x_coordinate, y_coordinate);
        }

        let drone = match monitor.get_drone(&id) {
            Some(drone) => drone,
            None => {
                println!("Drone not found");
                return;
            }
        };

        match monitor_sender.send(MonitorAction::DroneData(drone.clone())) {
            Ok(_) => println!("Drone data sent to UI"),
            Err(_) => println!("Error sending drone data to UI"),
        }
    }
}

fn handle_ui_action(
    stream: &Arc<Mutex<TcpStream>>,
    monitor: &Arc<Mutex<Monitor>>,
    action: UIAction,
) {
    let mut stream = stream.lock().unwrap();
    match action {
        UIAction::RegistrateDrone(drone_registration) => {
            let topic_name = TopicName::new(vec![DRONE_REGISTER.to_vec()], true);
            let message = drone_registration.build_drone_message().into_bytes();
            let dup = false;
            let qos = QoS::AtLeast;
            let retain = false;
            let package_identifier = Some(1);

            let publish = Publish::new(dup, qos, retain, topic_name, package_identifier, message);

            if let Err(e) = stream.write(publish.to_bytes().as_slice()) {
                println!("Error sending publish packet: {:?}", e);
            }
        }

        UIAction::RegistrateIncident(incident_registration) => {
            println!("Registering incident");

            let topic_name = TopicName::new(vec![NEW_INCIDENT.to_vec()], true);
            let message = incident_registration.build_incident_message().into_bytes();
            let dup = false;
            let qos = QoS::AtLeast;
            let retain = false;
            let package_identifier = Some(1);

            let publish = Publish::new(dup, qos, retain, topic_name, package_identifier, message);

            if let Err(e) = stream.write(publish.to_bytes().as_slice()) {
                println!("Error sending publish packet: {:?}", e);
            }

            let incident = Incident::new(
                incident_registration.name,
                incident_registration.description,
                incident_registration.x.parse::<f64>().unwrap(),
                incident_registration.y.parse::<f64>().unwrap(),
                "Open".to_string(),
            );

            let incident_clone = incident.clone();

            monitor.lock().unwrap().add_incident(incident_clone);

            // match monitor_sender.send(MonitorAction::IncidentData(incident.clone())) {
            //     Ok(_) => println!("Incident data sent to UI"),
            //     Err(_) => println!("Error sending incident data to UI"),
            // }
        }

        UIAction::ResolveIncident(incident) => {
            let topic_name = TopicName::new(vec![CLOSE_INCIDENT.to_vec()], true);
            let message = incident.build_incident_message().into_bytes();
            let dup = false;
            let qos = QoS::AtLeast;
            let retain = false;
            let package_identifier = Some(1);

            let publish = Publish::new(dup, qos, retain, topic_name, package_identifier, message);

            println!("Resolving incident");
            println!("{}", incident.build_incident_message());

            if let Err(e) = stream.write(publish.to_bytes().as_slice()) {
                println!("Error sending publish packet: {:?}", e);
            }
        }
    }
}
