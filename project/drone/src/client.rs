use std::{
    io::{ErrorKind, Read, Write},
    net::TcpStream,
    sync::{Arc, Mutex, MutexGuard},
    thread,
    time::Duration,
};

use mqtt::model::{
    components::{
        encoded_string::EncodedString, qos::QoS, topic_filter::TopicFilter,
        topic_level::TopicLevel, topic_name::TopicName,
    },
    packet::Packet,
    packets::{connect::Connect, publish::Publish, subscribe::Subscribe, unsubscribe::Unsubscribe},
};

use crate::{
    client,
    drone::Drone,
    drone_status::{DroneStatus, TravelLocation},
};

use common::incident::Incident;

const NEW_INCIDENT: &[u8] = b"new-incident";
const ATTENDING_INCIDENT: &[u8] = b"attending-incident";
const CLOSE_INCIDENT: &[u8] = b"close-incident";
const DRONE_DATA: &[u8] = b"drone-data";
const READY_INCIDENT: &[u8] = b"ready-incident";

const READ_MESSAGE_INTERVAL: u64 = 1;
const UPDATE_DATA_INTERVAL: u64 = 1;
const CHECK_BATTERY_INTERVAL: u64 = 5;

const TRAVEL_INTERVAL: u64 = 1;
const BATTERY_DISCHARGE_INTERVAL: u64 = 5;
const BATTERY_RECHARGE_INTERVAL: u64 = 1;

pub fn client_run(address: &str, client_id: &str) -> std::io::Result<()> {
    let server_stream = connect_to_server(address, client_id)?;
    let server_stream = Arc::new(Mutex::new(server_stream));

    let drone = Arc::new(Mutex::new(Drone::new()));

    let new_incident = TopicFilter::new(vec![TopicLevel::Literal(NEW_INCIDENT.to_vec())], false);

    match server_stream.lock() {
        Ok(mut server_stream) => {
            subscribe(new_incident, &mut server_stream)?;
        }
        Err(_) => {
            return Err(std::io::Error::new(ErrorKind::Other, "Mutex was poisoned"));
        }
    }

    let server_stream_clone = server_stream.clone();
    let drone_clone = drone.clone();

    let thread_update = thread::spawn(move || {
        update_drone_status(server_stream_clone, drone_clone);
    });

    let server_stream_cloned = server_stream.clone();
    let drone_cloned = drone.clone();

    let thread_read = thread::spawn(move || {
        read_incoming_packets(server_stream_cloned, drone_cloned);
    });

    let drone_cloned = drone.clone();
    let thread_discharge_battery = thread::spawn(move || {
        discharge_battery(drone_cloned);
    });

    let drone_cloned = drone.clone();
    let thread_recharge_battery = thread::spawn(move || {
        recharge_battery(drone_cloned);
    });

    thread_update.join().unwrap();
    thread_read.join().unwrap();
    thread_discharge_battery.join().unwrap();
    thread_recharge_battery.join().unwrap();

    Ok(())
}

fn read_incoming_packets(stream: Arc<Mutex<TcpStream>>, drone: Arc<Mutex<Drone>>) {
    loop {
        let mut buffer = [0; 1024];
        let mut locked_stream = stream.lock().unwrap();

        println!("Reading incoming packets");

        locked_stream.set_nonblocking(true).unwrap();
        match locked_stream.read(&mut buffer) {
            Ok(_) => {
                let packet = Packet::from_bytes(&mut buffer.as_slice()).unwrap();
                drop(locked_stream);

                match packet {
                    Packet::Publish(publish) => {
                        let cloned_drone = drone.clone();
                        let cloned_stream = stream.clone();

                        handle_publish(publish, cloned_drone, cloned_stream);
                    }
                    Packet::Puback(_) => println!("Received Puback packet!"),
                    Packet::Pingresp(_) => println!("Received Pingresp packet!"),
                    Packet::Suback(_) => println!("Received Suback packet!"),
                    Packet::Unsuback(_) => println!("Received Unsuback packet!"),
                    Packet::Pingreq(_) => println!("Received Pingreq packet!"),
                    Packet::Disconnect(_) => {
                        println!("Received Disconnect packet!");
                        break;
                    }
                    _ => println!("Received unsupported packet type"),
                }
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => {
                drop(locked_stream);
                thread::sleep(Duration::from_secs(READ_MESSAGE_INTERVAL));
                continue;
            }
            Err(e) => {
                println!("Lost connection to server: {:?}", e);
                break;
            }
        }
    }
}

fn handle_publish(
    publish: Publish,
    drone: Arc<Mutex<Drone>>,
    server_stream: Arc<Mutex<TcpStream>>,
) {
    let message = String::from_utf8(publish.message().to_vec()).unwrap();

    let topic_levels = publish.topic().levels();
    if topic_levels.len() == 1 && topic_levels[0] == NEW_INCIDENT {
        let incident = match Incident::from_string(message) {
            Ok(incident) => incident,
            Err(_) => {
                println!("Invalid incident message");
                return;
            }
        };
        handle_new_incident(incident, drone, server_stream);
        return;
    }

    if topic_levels.len() != 2 {
        return;
    }

    let incident_uuid = match String::from_utf8(topic_levels[1].to_vec()) {
        Ok(uuid) => uuid,
        Err(_) => {
            println!("Invalid incident uuid");
            return;
        }
    };

    if topic_levels[0] == ATTENDING_INCIDENT {
        handle_attending_incident(incident_uuid, drone, server_stream);
    } else if topic_levels[0] == READY_INCIDENT {
        //handle_ready_incident(incident_uuid, drone, server_stream);
    } else if topic_levels[0] == CLOSE_INCIDENT {
        handle_close_incident(incident_uuid, drone, server_stream)
    }
}

fn handle_new_incident(
    incident: Incident,
    drone: Arc<Mutex<Drone>>,
    server_stream: Arc<Mutex<TcpStream>>,
) {
    let topic_filter = TopicFilter::new(
        vec![
            TopicLevel::Literal(ATTENDING_INCIDENT.to_vec()),
            TopicLevel::Literal(incident.uuid.clone().into_bytes()),
        ],
        false,
    );

    let mut stream_locked = match server_stream.lock() {
        Ok(stream) => stream,
        Err(_) => {
            println!("Mutex was poisoned");
            return;
        }
    };

    match subscribe(topic_filter, &mut stream_locked) {
        Ok(_) => {
            println!("Subscribed to the incident topic");
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
            return;
        }
    }

    drop(stream_locked);

    let mut drone_locked = match drone.lock() {
        Ok(drone) => drone,
        Err(_) => {
            println!("Mutex was poisoned");
            return;
        }
    };

    if drone_locked.is_below_minimun() {
        println!("Drone battery is below minimum level");
        drop(drone_locked);
        return;
    }

    if !drone_locked.is_within_range(incident.x_coordinate, incident.y_coordinate) {
        println!("Drone is not within range of the incident");
        drop(drone_locked);
        return;
    }

    match drone_locked.status() {
        DroneStatus::Free => (),
        DroneStatus::Travelling(TravelLocation::Anchor) => (),
        _ => {
            println!("Drone is not free");
            drop(drone_locked);
            return;
        }
    }

    drone_locked.set_incident(Some(incident.clone()));
    drop(drone_locked);

    println!(
        "Traveling to incident location: {}, {}",
        incident.x_coordinate, incident.y_coordinate
    );

    let cloned_drone = drone.clone();

    let thread = thread::spawn(move || {
        travel(
            cloned_drone,
            incident.x_coordinate,
            incident.y_coordinate,
            TravelLocation::Incident,
        );
    });

    thread.join().unwrap();

    //publish_drone_arrived_incident(drone.clone(), &mut server_stream.lock().unwrap());

    let mut drone_locked = match drone.lock() {
        Ok(drone) => drone,
        Err(_) => {
            println!("Mutex was poisoned");
            return;
        }
    };

    if drone_locked.status() != DroneStatus::Travelling(TravelLocation::Incident) {
        println!("Drone is not attending the incident");

        let x = drone_locked.x_anchor_coordinate();
        let y = drone_locked.y_anchor_coordinate();
        drone_locked.set_incident(None);

        drop(drone_locked);

        travel(drone.clone(), x, y, TravelLocation::Central);

        drone_locked = match drone.lock() {
            Ok(drone) => drone,
            Err(_) => {
                println!("Mutex was poisoned");
                return;
            }
        };

        drone_locked.set_status(DroneStatus::Free);
        drop(drone_locked);
        return;
    }

    println!("Drone arrived to the incident location");
    drone_locked.set_status(DroneStatus::AttendingIncident);
    // drone_locked.increment_attending_counter();
    // println!("EL ATTENDING INCIDENT COUNT ES!!!!{}",drone_locked.attending_counter().to_string());

    drop(drone_locked);

    let mut locked_stream = match server_stream.lock() {
        Ok(stream) => stream,
        Err(_) => {
            println!("Mutex was poisoned");
            return;
        }
    };

    // let topic_filter = TopicFilter::new(
    //     vec![
    //         TopicLevel::Literal(ATTENDING_INCIDENT.to_vec()),
    //         TopicLevel::Literal(incident.uuid.clone().into_bytes()),
    //     ],
    //     false,
    // );

    // match unsubscribe(topic_filter, &mut locked_stream) {
    //     Ok(_) => println!("Unsubscribed from the incident topic"),
    //     Err(e) => eprintln!("Error: {:?}", e),
    // }

    let topic_name = TopicName::new(
        vec![
            TopicLevel::Literal(ATTENDING_INCIDENT.to_vec()).to_bytes(),
            TopicLevel::Literal(incident.uuid.clone().into_bytes()).to_bytes(),
        ],
        false,
    );
    let message = b"".to_vec();

    match publish(topic_name, message, &mut locked_stream, QoS::AtLeast) {
        Ok(_) => println!("Drone is attending the incident"),
        Err(e) => eprintln!("Error: {:?}", e),
    }

    let topic_filter = TopicFilter::new(
        vec![
            TopicLevel::Literal(CLOSE_INCIDENT.to_vec()),
            TopicLevel::Literal(incident.uuid.clone().into_bytes()),
        ],
        false,
    );

    match subscribe(topic_filter, &mut locked_stream) {
        Ok(_) => println!("Subscribed to the close incident topic"),
        Err(e) => eprintln!("Error: {:?}", e),
    }

    drop(locked_stream);
}

fn handle_attending_incident(
    uuid: String,
    drone: Arc<Mutex<Drone>>,
    server_stream: Arc<Mutex<TcpStream>>,
) {
    let mut drone_locked = match drone.lock() {
        Ok(drone) => drone,
        Err(_) => {
            println!("Mutex was poisoned");
            return;
        }
    };

    if drone_locked.incident().is_none() {
        println!("Drone has no incident assigned");
        return;
    }

    if drone_locked.status() == DroneStatus::AttendingIncident {
        let topic_filter = TopicFilter::new(
            vec![
                TopicLevel::Literal(CLOSE_INCIDENT.to_vec()),
                TopicLevel::Literal(uuid.clone().into_bytes()),
            ],
            false,
        );
        // como estoy en el lugar del incidente, me interesa saber cuando se cierra el incidente por el monitor
        // me subscribo a close incidente/uuid
        subscribe(topic_filter, &mut server_stream.lock().unwrap()).unwrap();
        println!("Subscribing to the close incident topic");

        let topic_filter = TopicFilter::new(
            vec![
                TopicLevel::Literal(ATTENDING_INCIDENT.to_vec()),
                TopicLevel::Literal(uuid.into_bytes()),
            ],
            false,
        );

        let mut locked_stream = match server_stream.lock() {
            Ok(stream) => stream,
            Err(_) => {
                println!("Mutex was poisoned");
                return;
            }
        };

        match unsubscribe(topic_filter, &mut locked_stream) {
            Ok(_) => println!("Unsubscribed from the attending incident topic"),
            Err(e) => eprintln!("Error: {:?}", e),
        };

        drop(locked_stream);

        println!("Unsubscribing from the incident topic");
    } else if drone_locked.attending_counter() == 0 {
        drone_locked.increment_attending_counter();
        println!("drone is attending the incident");

    // el caso que contando este drone, llegaron 2 drones a la misma locacion
    } else if drone_locked.attending_counter() == 1 {
        //drone_locked.increment_attending_counter();
        println!("TWO DRONES ATTENDING THE INCIDENT");
        let drone_cloned = drone.clone();
        let server_stream_clone = server_stream.clone();
        let uuid_clone = uuid.clone();

        let thread = thread::spawn(move || {
            simulate_incident_resolution(uuid_clone, drone_cloned, server_stream_clone);
        });

        thread.join().unwrap();

        let topic_filter = TopicFilter::new(
            vec![
                TopicLevel::Literal(CLOSE_INCIDENT.to_vec()),
                TopicLevel::Literal(uuid.into_bytes()),
            ],
            false,
        );

        let mut locked_stream = match server_stream.lock() {
            Ok(stream) => stream,
            Err(_) => {
                println!("Mutex was poisoned");
                return;
            }
        };

        match unsubscribe(topic_filter, &mut locked_stream) {
            Ok(_) => println!("Unsubscribed from the close incident topic"),
            Err(e) => eprintln!("Error: {:?}", e),
        }

        drop(locked_stream);
    }
    drop(drone_locked);
}

fn simulate_incident_resolution(
    uuid: String,
    drone: Arc<Mutex<Drone>>,
    server_stream: Arc<Mutex<TcpStream>>,
) {
    thread::sleep(Duration::from_secs(30));

    let topic_name = TopicName::new(
        vec![
            TopicLevel::Literal(READY_INCIDENT.to_vec()).to_bytes(),
            TopicLevel::Literal(uuid.into_bytes()).to_bytes(),
        ],
        false,
    );
    let message = b"".to_vec();
    let mut locked_stream = match server_stream.lock() {
        Ok(stream) => stream,
        Err(_) => {
            println!("Mutex was poisoned");
            return;
        }
    };
    match publish(topic_name, message, &mut locked_stream, QoS::AtLeast) {
        Ok(_) => println!("Incident has been resolved"),
        Err(e) => eprintln!("Error: {:?}", e),
    }
}

fn handle_close_incident(
    closing_incident_uuid: String,
    drone: Arc<Mutex<Drone>>,
    server_stream: Arc<Mutex<TcpStream>>,
) {
    let mut drone = match drone.lock() {
        Ok(drone) => drone,
        Err(_) => {
            println!("Mutex was poisoned");
            return;
        }
    };
    let current_incident = match drone.incident() {
        Some(current_incident) => current_incident,
        None => {
            println!("No current incident");
            return;
        }
    };

    if current_incident.uuid != closing_incident_uuid {
        println!("Close incident received does not match current incident of drone.");
        return;
    }

    drone.set_incident(None);
    println!("Current incident closed");

    let close_topic = format!("close-incident/{}", current_incident.uuid);
    let topic_filter = TopicFilter::new(
        vec![TopicLevel::Literal(close_topic.as_bytes().to_vec())],
        false,
    );

    let mut stream = server_stream.lock().unwrap();
    unsubscribe(topic_filter, &mut stream).unwrap();

    let x_anchor_point = drone.x_anchor_coordinate();
    let y_anchor_point = drone.y_anchor_coordinate();

    drone.travel_to(x_anchor_point, y_anchor_point);

    drop(drone);
}

fn update_drone_status(server_stream: Arc<Mutex<TcpStream>>, drone: Arc<Mutex<Drone>>) {
    loop {
        let drone = match drone.lock() {
            Ok(drone) => drone,
            Err(_) => {
                println!("Mutex was poisoned");
                return;
            }
        };

        let mut levels = vec![DRONE_DATA.to_vec()];
        levels.push(drone.id().to_string().into_bytes());

        let topic_name = TopicName::new(levels, false);
        let message = drone.data().into_bytes();

        drop(drone);
        let mut stream = match server_stream.lock() {
            Ok(server_stream) => server_stream,
            Err(_) => {
                println!("Mutex was poisoned");
                return;
            }
        };

        match publish(topic_name, message, &mut stream, QoS::AtMost) {
            Ok(_) => println!("Drone data updated"),
            Err(e) => eprintln!("Error: {:?}", e),
        }

        drop(stream);

        thread::sleep(Duration::from_secs(UPDATE_DATA_INTERVAL));
    }
}

fn connect_to_server(address: &str, client_id: &str) -> std::io::Result<TcpStream> {
    println!("\nConnecting to address: {:?}", address);
    let mut to_server_stream = TcpStream::connect(address)?;

    let client_id_bytes: Vec<u8> = client_id.as_bytes().to_vec();
    //let client_id_bytes: Vec<u8> = b"drone".to_vec();
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

fn subscribe(
    filter: TopicFilter,
    server_stream: &mut MutexGuard<TcpStream>,
) -> std::io::Result<()> {
    let mut server_stream = server_stream.try_clone().unwrap();

    let packet_id = 1;
    let qos = QoS::AtLeast;
    let topics_filters = vec![(filter, qos)];

    let subscribe_packet = Subscribe::new(packet_id, topics_filters);
    let _ = server_stream.write(subscribe_packet.to_bytes().as_slice());

    server_stream.set_nonblocking(false).unwrap();
    match Packet::from_bytes(&mut server_stream) {
        Ok(Packet::Suback(_)) => Ok(()),
        _ => Err(std::io::Error::new(
            ErrorKind::Other,
            "Suback was not received.",
        )),
    }
}

fn unsubscribe(
    filter: TopicFilter,
    server_stream: &mut MutexGuard<TcpStream>,
) -> std::io::Result<()> {
    let mut server_stream = server_stream.try_clone().unwrap();

    let packet_id = 1;
    let topics_filters = vec![(filter)];

    let unsubscribe_packet = Unsubscribe::new(packet_id, topics_filters);

    let _ = server_stream.write(unsubscribe_packet.to_bytes().as_slice());

    server_stream.set_nonblocking(false).unwrap();
    match Packet::from_bytes(&mut server_stream) {
        Ok(Packet::Unsuback(_)) => Ok(()),
        _ => Err(std::io::Error::new(
            ErrorKind::Other,
            "Unsuback was not received.",
        )),
    }
}

fn publish(
    topic_name: TopicName,
    message: Vec<u8>,
    server_stream: &mut MutexGuard<TcpStream>,
    qos: QoS,
) -> std::io::Result<()> {
    let mut server_stream = server_stream.try_clone().unwrap();

    let dup = false;
    let retain = false;
    let mut package_identifier = None;
    if qos == QoS::AtLeast {
        package_identifier = Some(1);
    } else if QoS::AtMost == qos {
        package_identifier = None;
    }
    let message_bytes = message;

    let publish_packet = Publish::new(
        dup,
        qos.clone(),
        retain,
        topic_name,
        package_identifier,
        message_bytes,
    );

    let _ = server_stream.write(publish_packet.to_bytes().as_slice());

    if qos == QoS::AtMost {
        return Ok(());
    }

    server_stream.set_nonblocking(false).unwrap();
    match Packet::from_bytes(&mut server_stream) {
        Ok(Packet::Puback(_)) => Ok(()),
        _ => Err(std::io::Error::new(
            ErrorKind::Other,
            "Puback was not received.???",
        )),
    }
}

fn travel(drone: Arc<Mutex<Drone>>, x: f64, y: f64, travel_location: TravelLocation) {
    println!("Traveling to ({}, {})", x, y);
    let mut locked_drone = match drone.lock() {
        Ok(drone) => drone,
        Err(_) => {
            println!("Mutex was poisoned");
            return;
        }
    };

    locked_drone.set_status(DroneStatus::Travelling(travel_location));
    drop(locked_drone);

    loop {
        let mut locked_drone = match drone.lock() {
            Ok(drone) => drone,
            Err(_) => {
                println!("Mutex was poisoned");
                return;
            }
        };
        let distance = locked_drone.distance_to(x, y);
        let status = locked_drone.status();
        println!("Distance to destination: {}", distance);

        if distance == 0.0 || status != DroneStatus::Travelling(travel_location) {
            drop(locked_drone);
            break;
        }

        locked_drone.travel_to(x, y);
        drop(locked_drone);
        thread::sleep(Duration::from_secs(TRAVEL_INTERVAL));
    }
}

fn discharge_battery(drone: Arc<Mutex<Drone>>) {
    loop {
        let mut locked_drone = match drone.lock() {
            Ok(drone) => drone,
            Err(_) => {
                println!("Mutex was poisoned");
                return;
            }
        };

        locked_drone.discharge_battery();
        println!("Drone battery level: {}", locked_drone.battery());
        drop(locked_drone);

        thread::sleep(Duration::from_secs(BATTERY_DISCHARGE_INTERVAL));
    }
}

fn recharge_battery(drone: Arc<Mutex<Drone>>) {
    loop {
        println!("Checking battery level");
        let locked_drone = match drone.lock() {
            Ok(drone) => drone,
            Err(_) => {
                println!("Mutex was poisoned");
                return;
            }
        };
        println!("Obtained lock");

        if !locked_drone.is_below_minimun() || locked_drone.status() != DroneStatus::Free {
            println!("Drone still has battery or is not free");
            drop(locked_drone);
            thread::sleep(Duration::from_secs(CHECK_BATTERY_INTERVAL));
            continue;
        }

        // RECHARGE BATTERY
        println!("RECHARGE BATTERY POR FAVOR");

        let x = locked_drone.x_central_coordinate();
        let y = locked_drone.y_central_coordinate();
        drop(locked_drone);

        travel(drone.clone(), x, y, TravelLocation::Central);

        let mut locked_drone = match drone.lock() {
            Ok(drone) => drone,
            Err(_) => {
                println!("Mutex was poisoned");
                return;
            }
        };

        println!("Drone rechargin battata");

        locked_drone.set_status(DroneStatus::Recharging);
        drop(locked_drone);

        loop {
            let mut locked_drone = match drone.lock() {
                Ok(drone) => drone,
                Err(_) => {
                    println!("Mutex was poisoned");
                    return;
                }
            };

            locked_drone.recharge_battery();
            if locked_drone.is_fully_charged() {
                drop(locked_drone);
                break;
            }
            drop(locked_drone);

            thread::sleep(Duration::from_secs(BATTERY_RECHARGE_INTERVAL));
        }

        let locked_drone = match drone.lock() {
            Ok(drone) => drone,
            Err(_) => {
                println!("Mutex was poisoned");
                return;
            }
        };

        let x = locked_drone.x_anchor_coordinate();
        let y = locked_drone.y_anchor_coordinate();
        drop(locked_drone);

        travel(drone.clone(), x, y, TravelLocation::Anchor);

        let mut locked_drone = match drone.lock() {
            Ok(drone) => drone,
            Err(_) => {
                println!("Mutex was poisoned");
                return;
            }
        };

        locked_drone.set_status(DroneStatus::Free);
        drop(locked_drone);
    }
}

// pub fn publish_drone_arrived_incident(drone: Arc<Mutex<Drone>>, server_stream: &mut MutexGuard<TcpStream>) {
//     let locked_drone = match drone.lock() {
//         Ok(drone) => drone,
//         Err(_) => {
//             println!("Mutex was poisoned");
//             return;
//         }
//     };

//     let mut levels = vec![DRONE_ARRIVED_INCIDENT.to_vec()];
//     levels.push(locked_drone.id().to_string().into_bytes());
//     levels.push(locked_drone.incident().unwrap().uuid.into_bytes());

//     drop(locked_drone);

//     let topic_name = TopicName::new(levels, false);

//     let message = b"".to_vec();
//     let qos = QoS::AtMost;

//     match publish(topic_name, message, server_stream, qos) {
//         Ok(_) => println!("Published drone arrived message"),
//         Err(e) => eprintln!("Error: {:?}", e),
//     }
// }
