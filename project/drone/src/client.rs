use std::{
    io::{ErrorKind, Write},
    net::TcpStream,
    sync::{Arc, Mutex, MutexGuard},
    thread,
    time::Duration,
};

use mqtt::model::{
    components::{
        encoded_string::EncodedString, login::Login, qos::QoS, topic_filter::TopicFilter,
        topic_level::TopicLevel, topic_name::TopicName,
    },
    packet::Packet,
    packets::{connect::Connect, publish::Publish, subscribe::Subscribe, unsubscribe::Unsubscribe},
    return_codes::connect_return_code::ConnectReturnCode,
};

use crate::{config::Config, drone::Drone};

use common::drone_status::{DroneStatus, TravelLocation};

use common::incident::Incident;

const NEW_INCIDENT: &[u8] = b"new-incident";
const ATTENDING_INCIDENT: &[u8] = b"attending-incident";
const CLOSE_INCIDENT: &[u8] = b"close-incident";
const DRONE_DATA: &[u8] = b"drone-data";
const READY_INCIDENT: &[u8] = b"ready-incident";

const READ_MESSAGE_INTERVAL: u64 = 100;
const UPDATE_DATA_INTERVAL: u64 = 1;
const CHECK_BATTERY_INTERVAL: u64 = 5;
const PENDING_INCIDENTS_INTERVAL: u64 = 1;
const WAIT_FOR_DRONE_INTERVAL: u64 = 1;

const TRAVEL_INTERVAL: u64 = 1;
const BATTERY_DISCHARGE_INTERVAL: u64 = 5;
const BATTERY_RECHARGE_INTERVAL: u64 = 1;

const DRONE_ATTENDING_DURATION: u64 = 10;

const DRONE_COUNT_PER_INCIDENT: usize = 2;

/// Runs the client with the specified configuration
pub fn client_run(config: Config) -> std::io::Result<()> {
    let server_stream = connect_to_server(config.clone())?;
    let server_stream = Arc::new(Mutex::new(server_stream));

    let drone = Arc::new(Mutex::new(Drone::new(
        config.get_id(),
        config.get_x_central_position(),
        config.get_y_central_position(),
        config.get_x_anchor_position(),
        config.get_y_anchor_position(),
        config.get_velocity(),
        config.get_active_range(),
    )));

    let key = config.get_key().to_owned();

    let new_incident = TopicFilter::new(vec![TopicLevel::Literal(NEW_INCIDENT.to_vec())], false);

    match server_stream.lock() {
        Ok(mut server_stream) => {
            subscribe(new_incident, &mut server_stream, &key)?;
        }
        Err(_) => {
            return Err(std::io::Error::new(ErrorKind::Other, "Mutex was poisoned"));
        }
    }

    let server_stream_clone = server_stream.clone();
    let drone_clone = drone.clone();

    let thread_update = thread::spawn(move || {
        update_drone_status(server_stream_clone, drone_clone, &key);
    });

    let server_stream_cloned = server_stream.clone();
    let drone_cloned = drone.clone();

    let thread_read = thread::spawn(move || {
        read_incoming_packets(server_stream_cloned, drone_cloned, &key);
    });

    // Thread to handle pending incidents
    let server_stream_cloned = server_stream.clone();
    let drone_cloned = drone.clone();

    let thread_pending_incidents = thread::spawn(move || {
        handle_pending_incidents(drone_cloned, server_stream_cloned, &key);
    });

    let drone_cloned = drone.clone();
    let thread_discharge_battery = thread::spawn(move || {
        discharge_battery(drone_cloned);
    });

    let drone_cloned = drone.clone();
    let thread_recharge_battery = thread::spawn(move || {
        recharge_battery(drone_cloned);
    });

    let x = config.get_x_anchor_position();
    let y = config.get_y_anchor_position();

    travel(drone.clone(), x, y, TravelLocation::Anchor);
    let mut locked_drone = match drone.lock() {
        Ok(drone) => drone,
        Err(_) => {
            println!("Mutex was poisoned");
            return Ok(());
        }
    };

    if locked_drone.is_in_anchor() {
        locked_drone.set_status(DroneStatus::Free);
    }

    drop(locked_drone);

    let threads = vec![
        thread_update,
        thread_read,
        thread_pending_incidents,
        thread_discharge_battery,
        thread_recharge_battery,
    ];

    for thread in threads {
        match thread.join() {
            Ok(_) => {}
            Err(_) => {
                println!("Error in thread");
            }
        }
    }

    Ok(())
}

/// Connects to the server with the specified address
fn connect_to_server(config: Config) -> std::io::Result<TcpStream> {
    let address = config.get_address();
    let id = config.get_id();
    let username = config.get_username();
    let password = config.get_password();
    let key = config.get_key();

    let mut to_server_stream = TcpStream::connect(address)?;

    let client_id_bytes: Vec<u8> = id.to_string().into_bytes();

    let client_id = EncodedString::new(client_id_bytes);
    let will = None;

    let username = EncodedString::from_string(&username.to_string());
    let password = Some(EncodedString::from_string(&password.to_string()));

    let login = Some(Login::new(username, password));
    let connect = Connect::new(false, 0, client_id, will, login);

    let _ = to_server_stream.write(connect.to_bytes(key).as_slice());

    match Packet::from_bytes(&mut to_server_stream, key) {
        Ok(Packet::Connack(connack)) => match connack.connect_return_code() {
            ConnectReturnCode::ConnectionAccepted => Ok(to_server_stream),
            _ => Err(std::io::Error::new(
                ErrorKind::Other,
                format!("Connection refused: {:?}", connack.connect_return_code()),
            )),
        },
        _ => Err(std::io::Error::new(ErrorKind::Other, "No connack recibed")),
    }
}

/// Reads incoming packets from the server
fn read_incoming_packets(stream: Arc<Mutex<TcpStream>>, drone: Arc<Mutex<Drone>>, key: &[u8; 32]) {
    loop {
        let locked_stream = match stream.lock() {
            Ok(stream) => stream,
            Err(_) => {
                println!("Mutex was poisoned");
                return;
            }
        };

        let mut cloned_stream = match locked_stream.try_clone() {
            Ok(stream) => stream,
            Err(_) => {
                println!("Mutex was poisoned");
                return;
            }
        };

        match cloned_stream.set_nonblocking(true) {
            Ok(_) => {}
            Err(_) => {
                println!("Error setting non-blocking");
                return;
            }
        }

        match Packet::from_bytes(&mut cloned_stream, key) {
            Ok(Packet::Publish(publish)) => {
                drop(locked_stream);
                let cloned_drone = drone.clone();
                let cloned_stream = stream.clone();

                handle_publish(publish, cloned_drone, cloned_stream, key);
                continue;
            }
            Ok(Packet::Puback(_)) => {}
            Ok(Packet::Pingresp(_)) => {}
            Ok(Packet::Suback(_)) => {}
            Ok(Packet::Unsuback(_)) => {}
            Ok(Packet::Pingreq(_)) => {}
            Ok(Packet::Disconnect(_)) => {
                break;
            }
            _ => {
                drop(locked_stream);
                thread::sleep(Duration::from_millis(READ_MESSAGE_INTERVAL));
                continue;
            }
        }

        drop(locked_stream);
    }
}

/// Handles the incoming publish packet
fn handle_publish(
    publish: Publish,
    drone: Arc<Mutex<Drone>>,
    server_stream: Arc<Mutex<TcpStream>>,
    key: &[u8; 32],
) {
    let message = match String::from_utf8(publish.message().to_vec()) {
        Ok(message) => message,
        Err(_) => {
            println!("Invalid message");
            return;
        }
    };
    let topic_levels = publish.topic().levels();
    let action = match topic_levels.get(0) {
        Some(action) => action.as_slice(),
        None => {
            println!("Invalid topic");
            return;
        }
    };

    match action {
        NEW_INCIDENT => handle_new_incident(message, drone),
        ATTENDING_INCIDENT | CLOSE_INCIDENT => {
            let uuid = match topic_levels.get(1) {
                Some(uuid) => match String::from_utf8(uuid.to_vec()) {
                    Ok(uuid) => uuid,
                    Err(_) => {
                        println!("Invalid incident uuid");
                        return;
                    }
                },
                None => {
                    println!("Invalid incident uuid");
                    return;
                }
            };

            match action {
                ATTENDING_INCIDENT => handle_attending_incident(uuid, drone),
                CLOSE_INCIDENT => handle_close_incident(uuid, drone, server_stream, key),
                _ => {}
            }
        }
        _ => {}
    }
}

/// Handles the new incident
fn handle_new_incident(message: String, drone: Arc<Mutex<Drone>>) {
    let incident = match Incident::from_string(message) {
        Ok(incident) => incident,
        Err(_) => {
            println!("Invalid incident message");
            return;
        }
    };

    let mut locked_drone = match drone.lock() {
        Ok(drone) => drone,
        Err(_) => {
            println!("Mutex was poisoned");
            return;
        }
    };

    if locked_drone.is_within_range(incident.x_coordinate, incident.y_coordinate) {
        locked_drone.add_incident(incident);
    }

    drop(locked_drone);
}

/// Handles the attending incident
fn handle_attending_incident(uuid: String, drone: Arc<Mutex<Drone>>) {
    let mut drone_locked = match drone.lock() {
        Ok(drone) => drone,
        Err(_) => {
            println!("Mutex was poisoned");
            return;
        }
    };

    let incident = match drone_locked.current_incident() {
        Some(incident) => incident,
        None => {
            return;
        }
    };

    if incident.uuid != uuid {
        return;
    }

    drone_locked.increment_attending_counter();

    if drone_locked.attending_counter() == DRONE_COUNT_PER_INCIDENT
        && drone_locked.status() != DroneStatus::AttendingIncident
    {
        drone_locked.set_status(DroneStatus::Interrupted);
    }

    drop(drone_locked);
}

/// Handles the closing of an incident
fn handle_close_incident(
    closing_incident_uuid: String,
    drone: Arc<Mutex<Drone>>,
    server_stream: Arc<Mutex<TcpStream>>,
    key: &[u8; 32],
) {
    let mut locked_drone = match drone.lock() {
        Ok(drone) => drone,
        Err(_) => {
            return;
        }
    };

    let current_incident = match locked_drone.current_incident() {
        Some(current_incident) => current_incident,
        None => {
            return;
        }
    };

    if current_incident.uuid != closing_incident_uuid {
        return;
    }

    // locked_drone.set_incident(None);
    locked_drone.remove_current_incident();

    let x = locked_drone.x_anchor_coordinate();
    let y = locked_drone.y_anchor_coordinate();

    drop(locked_drone);

    let topic_filter = TopicFilter::new(
        vec![
            TopicLevel::Literal(CLOSE_INCIDENT.to_vec()),
            TopicLevel::Literal(current_incident.uuid.clone().into_bytes()),
        ],
        false,
    );

    let mut stream = match server_stream.lock() {
        Ok(stream) => stream,
        Err(_) => {
            println!("Mutex was poisoned");
            return;
        }
    };

    match unsubscribe(topic_filter, &mut stream, key) {
        Ok(_) => {}
        Err(e) => eprintln!("Error: {:?}", e),
    }

    drop(stream);

    thread::spawn(move || {
        travel(drone.clone(), x, y, TravelLocation::Anchor);

        let mut locked_drone = match drone.lock() {
            Ok(drone) => drone,
            Err(_) => {
                println!("Mutex was poisoned");
                return;
            }
        };

        if locked_drone.is_in_anchor() {
            locked_drone.set_status(DroneStatus::Free);
        }

        drop(locked_drone);
    });
}

/// Updates the drone status
fn update_drone_status(
    server_stream: Arc<Mutex<TcpStream>>,
    drone: Arc<Mutex<Drone>>,
    key: &[u8; 32],
) {
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

        match publish(topic_name, message, &mut stream, QoS::AtMost, key) {
            Ok(_) => {}
            Err(e) => eprintln!("Error: {:?}", e),
        }

        drop(stream);

        thread::sleep(Duration::from_secs(UPDATE_DATA_INTERVAL));
    }
}

/// Subscribes to the specified topic filter
fn subscribe(
    filter: TopicFilter,
    server_stream: &mut MutexGuard<TcpStream>,
    key: &[u8; 32],
) -> std::io::Result<()> {
    let mut server_stream = match server_stream.try_clone() {
        Ok(stream) => stream,
        Err(_) => {
            println!("Mutex was poisoned");
            return Err(std::io::Error::new(ErrorKind::Other, "Mutex was poisoned"));
        }
    };

    let packet_id = 1;
    let qos = QoS::AtMost;
    let topics_filters = vec![(filter, qos)];

    let subscribe_packet = Subscribe::new(packet_id, topics_filters);
    let _ = server_stream.write(subscribe_packet.to_bytes(key).as_slice());

    Ok(())
}

/// Unsubscribes from the specified topic filter
fn unsubscribe(
    filter: TopicFilter,
    server_stream: &mut MutexGuard<TcpStream>,
    key: &[u8; 32],
) -> std::io::Result<()> {
    let mut server_stream = match server_stream.try_clone() {
        Ok(stream) => stream,
        Err(_) => {
            println!("Mutex was poisoned");
            return Err(std::io::Error::new(ErrorKind::Other, "Mutex was poisoned"));
        }
    };

    let packet_id = 1;
    let topics_filters = vec![(filter)];

    let unsubscribe_packet = Unsubscribe::new(packet_id, topics_filters);

    let _ = server_stream.write(unsubscribe_packet.to_bytes(key).as_slice());

    Ok(())
}

/// Publishes the specified message to the server
fn publish(
    topic_name: TopicName,
    message: Vec<u8>,
    server_stream: &mut MutexGuard<TcpStream>,
    qos: QoS,
    key: &[u8; 32],
) -> std::io::Result<()> {
    let mut server_stream = match server_stream.try_clone() {
        Ok(stream) => stream,
        Err(_) => {
            println!("Mutex was poisoned");
            return Err(std::io::Error::new(ErrorKind::Other, "Mutex was poisoned"));
        }
    };

    let dup = false;
    let retain = true;
    let package_identifier = None;
    let message_bytes = message;

    let publish_packet = Publish::new(
        dup,
        qos.clone(),
        retain,
        topic_name,
        package_identifier,
        message_bytes,
    );

    let _ = server_stream.write(publish_packet.to_bytes(key).as_slice());

    Ok(())
}

/// Travels to the specified location
fn travel(drone: Arc<Mutex<Drone>>, x: f64, y: f64, travel_location: TravelLocation) {
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

        if distance == 0.0 || status != DroneStatus::Travelling(travel_location) {
            drop(locked_drone);
            break;
        }

        locked_drone.travel_to(x, y);
        drop(locked_drone);
        thread::sleep(Duration::from_secs(TRAVEL_INTERVAL));
    }
}

/// Handles the pending incidents of the drone queue
fn handle_pending_incidents(
    drone: Arc<Mutex<Drone>>,
    server_stream: Arc<Mutex<TcpStream>>,
    key: &[u8; 32],
) {
    loop {
        let locked_drone = match drone.lock() {
            Ok(drone) => drone,
            Err(_) => {
                println!("Mutex was poisoned");
                continue;
            }
        };

        if locked_drone.is_below_minimun() {
            drop(locked_drone);
            thread::sleep(Duration::from_secs(PENDING_INCIDENTS_INTERVAL));
            continue;
        }

        if !locked_drone.is_free() {
            drop(locked_drone);
            thread::sleep(Duration::from_secs(PENDING_INCIDENTS_INTERVAL));
            continue;
        }

        match locked_drone.current_incident() {
            Some(incident) => {
                drop(locked_drone);

                let drone = drone.clone();
                let server_stream = server_stream.clone();
                let key = *key;

                thread::spawn(move || {
                    handle_incident(incident, drone.clone(), server_stream.clone(), &key);
                });
            }
            None => {
                drop(locked_drone);
            }
        }
        thread::sleep(Duration::from_secs(PENDING_INCIDENTS_INTERVAL));
    }
}

/// Handles the last incident
fn handle_incident(
    incident: Incident,
    drone: Arc<Mutex<Drone>>,
    server_stream: Arc<Mutex<TcpStream>>,
    key: &[u8; 32],
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

    match subscribe(topic_filter, &mut stream_locked, key) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error: {:?}", e);
        }
    }

    drop(stream_locked);

    travel(
        drone.clone(),
        incident.x_coordinate,
        incident.y_coordinate,
        TravelLocation::Incident,
    );

    let mut drone_locked = match drone.lock() {
        Ok(drone) => drone,
        Err(_) => {
            println!("Mutex was poisoned");
            return;
        }
    };

    if drone_locked.is_interrupted() {
        drone_locked.remove_current_incident();

        let x = drone_locked.x_anchor_coordinate();
        let y = drone_locked.y_anchor_coordinate();
        drop(drone_locked);

        travel(drone.clone(), x, y, TravelLocation::Anchor);

        let mut locked_drone = match drone.lock() {
            Ok(drone) => drone,
            Err(_) => {
                println!("Mutex was poisoned");
                return;
            }
        };
        if locked_drone.is_in_anchor() {
            locked_drone.set_status(DroneStatus::Free);
        }

        drop(locked_drone);

        return;
    }

    drone_locked.set_status(DroneStatus::AttendingIncident);
    drop(drone_locked);

    let mut locked_stream = match server_stream.lock() {
        Ok(stream) => stream,
        Err(_) => {
            println!("Mutex was poisoned");
            return;
        }
    };

    let topic_name = TopicName::new(
        vec![
            TopicLevel::Literal(ATTENDING_INCIDENT.to_vec()).to_bytes(),
            TopicLevel::Literal(incident.uuid.clone().into_bytes()).to_bytes(),
        ],
        false,
    );
    let message = b"".to_vec();

    match publish(topic_name, message, &mut locked_stream, QoS::AtMost, key) {
        Ok(_) => {}
        Err(_) => println!("Drone is attending the incident. no le llego el puback"),
    }

    drop(locked_stream);

    loop {
        let locked_drone = match drone.lock() {
            Ok(drone) => drone,
            Err(_) => {
                println!("Mutex was poisoned");
                return;
            }
        };

        if locked_drone.attending_counter() >= DRONE_COUNT_PER_INCIDENT {
            drop(locked_drone);
            break;
        }

        drop(locked_drone);
        thread::sleep(Duration::from_secs(WAIT_FOR_DRONE_INTERVAL));
    }

    let mut locked_stream = match server_stream.lock() {
        Ok(stream) => stream,
        Err(_) => {
            println!("Mutex was poisoned");
            return;
        }
    };

    let topic_filter = TopicFilter::new(
        vec![
            TopicLevel::Literal(CLOSE_INCIDENT.to_vec()),
            TopicLevel::Literal(incident.uuid.clone().into_bytes()),
        ],
        false,
    );

    match subscribe(topic_filter, &mut locked_stream, key) {
        Ok(_) => {}
        Err(_) => println!("Drone subscribe to close incident topic. no le llego el suback"),
    }

    let topic_filter = TopicFilter::new(
        vec![
            TopicLevel::Literal(ATTENDING_INCIDENT.to_vec()),
            TopicLevel::Literal(incident.uuid.clone().into_bytes()),
        ],
        false,
    );

    match unsubscribe(topic_filter, &mut locked_stream, key) {
        Ok(_) => {}
        Err(e) => eprintln!("Error: {:?}", e),
    }

    drop(locked_stream);

    let duration_incident = Duration::from_secs(DRONE_ATTENDING_DURATION);

    thread::sleep(duration_incident);

    let topic_name = TopicName::new(
        vec![
            TopicLevel::Literal(READY_INCIDENT.to_vec()).to_bytes(),
            TopicLevel::Literal(incident.uuid.into_bytes()).to_bytes(),
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

    match publish(topic_name, message, &mut locked_stream, QoS::AtMost, key) {
        Ok(_) => {}
        Err(e) => eprintln!("Error: {:?}", e),
    }

    drop(locked_stream);
}

/// Discharges the battery of the drone
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
        drop(locked_drone);

        thread::sleep(Duration::from_secs(BATTERY_DISCHARGE_INTERVAL));
    }
}

/// Recharges the battery of the drone
fn recharge_battery(drone: Arc<Mutex<Drone>>) {
    loop {
        let locked_drone = match drone.lock() {
            Ok(drone) => drone,
            Err(_) => {
                println!("Mutex was poisoned");
                return;
            }
        };

        if !locked_drone.is_below_minimun() || !locked_drone.is_free() {
            drop(locked_drone);
            thread::sleep(Duration::from_secs(CHECK_BATTERY_INTERVAL));
            continue;
        }

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

        if locked_drone.is_in_anchor() {
            locked_drone.set_status(DroneStatus::Free);
        }

        drop(locked_drone);
    }
}
