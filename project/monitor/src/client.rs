use std::{
    collections::HashMap,
    io::{ErrorKind, Write},
    net::TcpStream,
    sync::mpsc::{channel, Receiver, Sender},
};

use common::incident::{Incident, IncidentStatus};
use mqtt::model::{
    components::{
        encoded_string::EncodedString, qos::QoS, topic_filter::TopicFilter,
        topic_level::TopicLevel, topic_name::TopicName,
    },
    packet::Packet,
    packets::{connect::Connect, puback::Puback, publish::Publish, subscribe::Subscribe},
};

use crate::{
    camera::Camera,
    channels_tasks::{DroneRegistration, IncidentRegistration, MonitorAction, UIAction},
    drone::Drone,
    monitor::Monitor,
    ui_application::UIApplication,
};


pub fn client_run(address: String) -> Result<(), String> {
    // Create the channels to communicate between the monitor and the UI
    let (monitor_sender, monitor_receiver) = channel();
    let (ui_sender, ui_receiver) = channel();

    // Connect to the server
    let mut stream = match connect_to_server(address) {
        Ok(stream) => stream,
        Err(e) => {
            return Err(format!("Error connecting to server: {:?}", e));
        }
    };

    // Subscribe to the topics
    match subscribe_to_topics(&mut stream) {
        Ok(_) => {}
        Err(e) => {
            return Err(format!("Error subscribing to topics: {:?}", e));
        }
    }

    // monitor start in a thread to avoid blocking the main thread
    let monitor_thread = std::thread::spawn(move || {
        start_monitor(stream, monitor_sender, ui_receiver);
    });

    // start the ui in the main thread
    match start_ui(ui_sender, monitor_receiver) {
        Ok(_) => {}
        Err(err) => {
            println!("Error starting UI: {:?}", err);
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
        _ => Err(std::io::Error::new(ErrorKind::Other, "No connack recibed")),
    }
}

// static CLIENT_ARGS: usize = 2;

// impl Client {
//     pub fn new(sender: Sender<String>) -> Self {
//         let argv = env::args().collect::<Vec<String>>();
//         if argv.len() != CLIENT_ARGS {
//             let app_name = &argv[0];
//             println!("Usage:\n{:?} <toml file>", app_name);main
//         }
//         _ => Err(std::io::Error::new(ErrorKind::Other, "No connack recibed")),
//     }
// }

fn start_ui(
    ui_sender: Sender<UIAction>,
    from_monitor_receiver: Receiver<MonitorAction>,
) -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        ..Default::default()
    };
        // let path = Path::new(&argv[1]);

        // let config = match Config::from_file(path) {
        //     Ok(config) => config,
        //     Err(e) => {
        //         println!("Error reading the configuration file: {:?}", e);
        //         std::process::exit(1);
        //     }
        // };

        // let address = config.get_address().to_owned();


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

const CAMERA_DATA: &[u8] = b"camera-data";
const DRONE_DATA: &[u8] = b"drone-data";
const DRONE_REGISTER: &[u8] = b"$drone-register";
const NEW_INCIDENT: &[u8] = b"new-incident";
const ATTENDING_INCIDENT: &[u8] = b"attending-incident";
const READY_INCIDENT: &[u8] = b"ready-incident";
const CLOSE_INCIDENT: &[u8] = b"close-incident";

const SEPARATOR: char = ';';
const ENUMARATOR: char = '|';

fn start_monitor(
    stream: TcpStream,
    monitor_sender: Sender<MonitorAction>,
    ui_reciver: Receiver<UIAction>,
) {
    let mut monitor = Monitor::new();
    let mut unacknowledged_publish = HashMap::new();
    let mut publish_counter = 0;

    let mut stream = stream;

    match stream.set_nonblocking(true) {
        Ok(_) => {}
        Err(_) => {
            println!("Error setting stream to non-blocking");
        }
    }

    loop {
        match Packet::from_bytes(&mut stream) {
            Ok(Packet::Puback(puback)) => {
                let packet_id = puback.packet_identifier();

                if unacknowledged_publish.remove(&packet_id).is_none() {
                    println!("Publish id does not match the puback id");
                }
            }
            Ok(Packet::Publish(publish)) => {
                let topic_name = publish.topic();
                let topic_levels = topic_name.levels();

                match topic_levels[0].as_slice() {
                    DRONE_DATA => {
                        drone_data(publish.clone(), monitor_sender.clone());
                    }
                    CAMERA_DATA => {
                        camera_data(publish.clone(), monitor_sender.clone());
                    }
                    ATTENDING_INCIDENT => {
                        attend_incident(publish.clone(), &mut monitor, monitor_sender.clone());
                    }
                    READY_INCIDENT => {
                        println!("LE LLEGO UN READY INCIDENT AL MONITOR");
                        ready_incident(publish.clone(), &mut monitor, monitor_sender.clone());
                    }
                    _ => {
                        println!("Unknown topic");
                    }
                }

                if publish.qos() == &QoS::AtLeast {
                    let package_identifier = publish.package_identifier();

                    let puback = Puback::new(package_identifier);

                    match stream.write(puback.to_bytes().as_slice()) {
                        Ok(_) => {}
                        Err(_) => {
                            println!("Error sending puback packet");
                        }
                    }
                }
            }

            Ok(_) => {}
            Err(_) => {} // println!("Error reading packet from server");
        }

        let publish = match ui_reciver.try_recv() {
            Ok(UIAction::RegistrateDrone(drone_registration)) => {
                register_drone(drone_registration, publish_counter)
            }

            Ok(UIAction::RegistrateIncident(incident_registration)) => register_incident(
                incident_registration,
                &mut monitor,
                monitor_sender.clone(),
                publish_counter,
            ),

            Ok(UIAction::ResolveIncident(incident)) => resolve_incident(incident, publish_counter),
            Err(_) => None,
        };

        if let Some(publish) = publish {
            match stream.write(publish.to_bytes().as_slice()) {
                Ok(_) => {
                    unacknowledged_publish.insert(publish.package_identifier(), publish.clone());
                }
                Err(_) => {
                    println!("Error sending publish packet");
                }
            }

            publish_counter += 1;
        }
    }
}

fn drone_data(publish: Publish, monitor_sender: Sender<MonitorAction>) {
    let topic_name = publish.topic();
    let topic_levels = topic_name.levels();

    let id = topic_levels[1].as_slice();

    let id = String::from_utf8_lossy(id.to_vec().as_slice()).to_string();
    let content = publish.message();
    let content_str = std::str::from_utf8(content).unwrap();
    let splitted_content: Vec<&str> = content_str.split(SEPARATOR).collect();

    let x_coordinate = splitted_content[0].parse::<f64>().unwrap();
    let y_coordinate = splitted_content[1].parse::<f64>().unwrap();
    let state = splitted_content[2].to_string();
    let battery = splitted_content[3].parse::<usize>().unwrap();

    let drone = Drone::new(id.clone(), state, battery, x_coordinate, y_coordinate);

    match monitor_sender.send(MonitorAction::Drone(drone.clone())) {
        Ok(_) => {
            // println!("Drone data sent to UI");
        }
        Err(_) => {
            println!("Error sending drone data to UI");
        }
    }
}

fn camera_data(publish: Publish, monitor_sender: Sender<MonitorAction>) {
    let content = publish.message();

    let content_str = std::str::from_utf8(content).unwrap();
    let splitted_content: Vec<&str> = content_str.split(ENUMARATOR).collect();

    // this are camera data
    for camera_data in splitted_content {
        let camera_data = camera_data.split(SEPARATOR).collect::<Vec<&str>>();
        let id = camera_data[0].parse::<String>().unwrap();
        let x_coordinate = camera_data[1].parse::<f64>().unwrap();
        let y_coordinate = camera_data[2].parse::<f64>().unwrap();
        let state = camera_data[3].parse::<String>().unwrap();

        let camera = Camera::new(id, x_coordinate, y_coordinate, state);

        match monitor_sender.send(MonitorAction::Camera(camera)) {
            Ok(_) => {
                println!("Camera data sent to UI");
            }
            Err(_) => {
                println!("Error sending camera data to UI");
            }
        }
    }
}

fn attend_incident(publish: Publish, monitor: &mut Monitor, monitor_sender: Sender<MonitorAction>) {
    let topic_name = publish.topic();
    let topic_levels = topic_name.levels();
    let incident_id = topic_levels[1].as_slice();
    let incident_id = String::from_utf8_lossy(incident_id.to_vec().as_slice()).to_string();

    if let Some(incident) = monitor.attend_incident(incident_id.clone()) {
        match monitor_sender.send(MonitorAction::Incident(incident)) {
            Ok(_) => {}
            Err(_) => {
                println!("Error sending incident data to UI");
            }
        }
    } else {
        println!("Unknown incident");
    }
}

fn ready_incident(publish: Publish, monitor: &mut Monitor, monitor_sender: Sender<MonitorAction>) {
    let topic_name = publish.topic();
    let topic_levels = topic_name.levels();
    let incident_id = topic_levels[1].as_slice();
    let incident_id = String::from_utf8_lossy(incident_id.to_vec().as_slice()).to_string();
    monitor.set_resolvable_incident(incident_id.clone());

    if let Some(incident) = monitor.get_incident(incident_id.as_str()) {
        println!("ENTRO AL IF DE SI OBTUVE EL INCIDENTE CO EL GET? ");
        match monitor_sender.send(MonitorAction::Incident(incident.clone())) {
            Ok(_) => {}
            Err(_) => {
                println!("Error sending incident data to UI");
            }
        }
    } else {
        println!("Unknown incident");
    }
}

fn register_drone(
    drone_registration: DroneRegistration,
    package_identifier: u16,
) -> Option<Publish> {
    let topic_name = TopicName::new(vec![DRONE_REGISTER.to_vec()], true);
    let message = drone_registration.build_drone_message().into_bytes();
    let dup = false;
    let qos = QoS::AtLeast;
    let retain = false;
    let package_identifier = Some(package_identifier);

    Some(Publish::new(
        dup,
        qos,
        retain,
        topic_name,
        package_identifier,
        message,
    ))
}

fn register_incident(
    incident_registration: IncidentRegistration,
    monitor: &mut Monitor,
    monitor_sender: Sender<MonitorAction>,
    package_identifier: u16,
) -> Option<Publish> {
    let incident = Incident::new(
        incident_registration.name.clone(),
        incident_registration.name.clone(),
        incident_registration.description.clone(),
        incident_registration.x.clone().parse().unwrap(),
        incident_registration.y.clone().parse().unwrap(),
        IncidentStatus::Pending,
    );

    let topic_name = TopicName::new(vec![NEW_INCIDENT.to_vec()], false);
    let message = incident.to_string().into_bytes();
    let dup = false;
    let qos = QoS::AtLeast;
    let retain = false;
    let package_identifier = Some(package_identifier);

    let publish = Publish::new(dup, qos, retain, topic_name, package_identifier, message);

    monitor.new_incident(incident.clone());

    match monitor_sender.send(MonitorAction::Incident(incident.clone())) {
        Ok(_) => Some(publish),
        Err(_) => None,
    }
}

fn resolve_incident(incident: Incident, package_identifier: u16) -> Option<Publish> {
    let topic_name = TopicName::new(
        vec![CLOSE_INCIDENT.to_vec(), incident.uuid.clone().into_bytes()],
        false,
    );
    let message = vec![];
    let dup = false;
    let qos = QoS::AtLeast;
    let retain = false;
    let package_identifier = Some(package_identifier);

    Some(Publish::new(
        dup,
        qos,
        retain,
        topic_name,
        package_identifier,
        message,
    ))
}

fn subscribe_to_topics(stream: &mut TcpStream) -> std::io::Result<()> {
    let mut topic_filters = vec![];

    let topics = vec![
        "camera-data",
        "camera-update",
        "attending-incident/+",
        "drone-data/+",
        "ready-incident/+",
    ];

    for topic in topics {
        let mut levels = vec![];
        for level in topic.split('/') {
            if let Ok(topic_level) = TopicLevel::from_bytes(level.as_bytes().to_vec()) {
                levels.push(topic_level);
            }
        }

        let topic_filter = TopicFilter::new(levels, false);
        let qos = QoS::AtLeast;

        topic_filters.push((topic_filter, qos));
    }

    let subscribe = Subscribe::new(1, topic_filters);

    match stream.write(subscribe.to_bytes().as_slice()) {
        Ok(_) => {}
        Err(_) => {
            return Err(std::io::Error::new(
                ErrorKind::Other,
                "Error sending subscribe packet",
            ));
        }
    }

    match Packet::from_bytes(stream) {
        Ok(Packet::Suback(_)) => Ok(()),
        _ => Err(std::io::Error::new(
            ErrorKind::Other,
            "Suback was not received.",
        )),
    }
}