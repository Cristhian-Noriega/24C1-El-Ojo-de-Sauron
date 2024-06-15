use std::{
    collections::HashMap, io::{ErrorKind, Write}, net::TcpStream, sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    }
};

use mqtt::model::{
    components::{encoded_string::EncodedString, qos::QoS, topic_level::TopicLevel, topic_name::TopicName},
    packet::Packet,
    packets::{connect::Connect, puback::Puback, publish::Publish},
};

use crate::{
    channels_tasks::{MonitorAction, UIAction}, drone::Drone, monitor::Monitor, ui_application::UIApplication
};

pub fn client_run(address: String) -> Result<(), String> {
    let (monitor_sender, monitor_receiver) = channel();
    let (ui_sender, ui_receiver) = channel();

    let stream = match connect_to_server(address) {
        Ok(stream) => stream,
        Err(e) => {
            return Err(format!("Error connecting to server: {:?}", e));
        }
    };

    let monitor_thread = std::thread::spawn(move || {
        start_monitor(
            stream,
            monitor_sender,
            ui_receiver
        );
    });

    match start_ui(ui_sender, monitor_receiver) {
        Ok(_) => {}
        Err(_) => {
            return Err("Error starting UI".to_string());
        }
    }

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
const SEPARATOR: char = ';';    
const CAMERA_DATA: &[u8] = b"camera-data";

fn start_monitor(
    stream: TcpStream,
    monitor_sender: Sender<MonitorAction>,
    ui_reciver: Receiver<UIAction>,
) {

    let mut monitor = Monitor::new();
    let mut unacknowledged_publish = HashMap::new();
    let mut publish_counter = 0;

    let mut stream = stream;

    stream.set_nonblocking(true);

    loop {
        match Packet::from_bytes(&mut stream) {
            Ok(Packet::Puback(puback)) => {
                let packet_id = puback.packet_identifier();
                
                if unacknowledged_publish.remove(&packet_id).is_none() {
                    println!("Publish id does not match the puback id");
                }
            }
            Ok(Packet::Publish(publish)) => {
                unacknowledged_publish.insert(publish.package_identifier(), publish.clone());
                
                let topic_name = publish.topic();

                let topic_levels = topic_name.levels();

                if topic_levels.len() == 2 && topic_levels[0] == DRONE_DATA {

                    let id = topic_levels[1].as_slice();

                    let id = String::from_utf8_lossy(id.to_vec().as_slice()).to_string();
                    let content = publish.message();
                    let content_str = std::str::from_utf8(&content).unwrap();
                    let splitted_content: Vec<&str> = content_str.split(SEPARATOR).collect();

                    let x_coordinate = splitted_content[0].parse::<f64>().unwrap();
                    let y_coordinate = splitted_content[1].parse::<f64>().unwrap();
                    let state = splitted_content[2].to_string();
                    let battery = splitted_content[3].parse::<usize>().unwrap();

                    //let id_cloned = id.clone();

                    if !monitor.has_registered_drone(&id) {

                        let drone = Drone::new(
                            id.clone(),
                            state,
                            battery,
                            x_coordinate,
                            y_coordinate,
                        );

                        monitor.add_drone(drone)
                    } else {
                        monitor.update_drone(&id, state, battery, x_coordinate, y_coordinate);
                    }

                    let drone = match monitor.get_drone(&id) {
                        Some(drone) => drone,
                        None => {
                            println!("Drone not found");
                            continue;
                        }
                    };

                    match monitor_sender.send(MonitorAction::DroneData(drone.clone())) {
                        Ok(_) => {}
                        Err(_) => {
                            println!("Error sending drone data to UI");
                        }
                    }
                }


            }
            Ok(_) => {}
            // CAMERA DATA
            // ATTENDING INCIDENT

            Err(_) => {
                println!("Error reading packet from server");
            }
        }

        match ui_reciver.try_recv() {
            Ok(UIAction::RegistrateDrone(drone_registration)) => {
                let drone_id = drone_registration.id;
                let password = drone_registration.password;
                let anchor_coords_x = drone_registration.anchor_x;
                let anchor_coords_y = drone_registration.anchor_y;

                let topic_name = TopicName::new(vec![CAMERA_DATA.to_vec()], true);
                let message = format!("{};{};{};{}", drone_id, password, anchor_coords_x, anchor_coords_y);
                let dup = false;
                let qos = QoS::AtLeast;
                let retain = false;
                let package_identifier = Some(1);

                let publish = Publish::new(dup, qos, retain, topic_name, package_identifier, message.as_bytes().to_vec());

                publish_counter += 1;

                unacknowledged_publish.insert(Some(publish_counter), publish.clone());

                match stream.write(publish.to_bytes().as_slice()) {
                    Ok(_) => {}
                    Err(_) => {
                        println!("Error sending publish packet");
                    }
                }
            }

            Ok(UIAction::StartIncident(incident)) => {
                let topic_name = TopicName::new(vec![NEW_INCIDENT.to_vec()], true);
                let message = incident;
                let dup = false;
                let qos = QoS::AtLeast;
                let retain = false;
                let package_identifier = Some(1);

                let publish = Publish::new(dup, qos, retain, topic_name, package_identifier, message.as_bytes().to_vec());

                publish_counter += 1;

                unacknowledged_publish.insert(Some(publish_counter), publish.clone());

                match stream.write(publish.to_bytes().as_slice()) {
                    Ok(_) => {}
                    Err(_) => {
                        println!("Error sending publish packet");
                    }
                }


            }

            Ok(UIAction::RegistrateIncident(incident_registration)) => {
                let name = incident_registration.name;
                let description = incident_registration.description;
                let x = incident_registration.x;
                let y = incident_registration.y;

                let topic_name = TopicName::new(vec![NEW_INCIDENT.to_vec()], true);
                let message = format!("{};{};{};{}", name, description, x, y);
                let dup = false;
                let qos = QoS::AtLeast;
                let retain = false;
                let package_identifier = Some(1);

                let publish = Publish::new(dup, qos, retain, topic_name, package_identifier, message.as_bytes().to_vec());

                publish_counter += 1;

                unacknowledged_publish.insert(Some(publish_counter), publish.clone());

                match stream.write(publish.to_bytes().as_slice()) {
                    Ok(_) => {}
                    Err(_) => {
                        println!("Error sending publish packet");
                    }
                }
            }

            Err(_) => {

            }
        }
    }
}



