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

use crate::{drone::Drone, drone_status::DroneStatus, incident::Incident};

const NEW_INCIDENT: &[u8] = b"new-incident";
const ATTENDING_INCIDENT: &[u8] = b"attending-incident";
const CLOSE_INCIDENT: &[u8] = b"close-incident";
const DRONE_DATA: &[u8] = b"drone-data";

const UPDATE_INTERVAL: u64 = 5000;

pub fn client_run(address: &str) -> std::io::Result<()> {
    let server_stream = connect_to_server(address)?;
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

    // let publish = Publish::new(
    //     false,
    //     QoS::AtLeast,
    //     false,
    //     TopicName::new(
    //         vec![TopicLevel::Literal(NEW_INCIDENT.to_vec()).to_bytes()],
    //         false,
    //     ),
    //     None,
    //     b"1;Incidente 1;Incidente de prueba;10.0;10.0;0".to_vec(),
    // );

    // handle_publish(publish, drone.clone(), server_stream.clone());

    // JUST FOR TESTING
    // travel(drone.clone(), 10.0, 10.0);

    thread_update.join().unwrap();
    thread_read.join().unwrap();

    Ok(())
}

fn read_incoming_packets(server_stream: Arc<Mutex<TcpStream>>, drone: Arc<Mutex<Drone>>) {
    loop {
        let mut buffer = [0; 1024];
        let mut stream = server_stream.lock().unwrap();

        println!("Reading incoming packets");

        stream.set_nonblocking(true).unwrap();
        match stream.read(&mut buffer) {
            Ok(_) => {
                let packet = Packet::from_bytes(&mut buffer.as_slice()).unwrap();
                match packet {
                    Packet::Publish(publish) => {
                        let cloned_drone = drone.clone();
                        let cloned_stream = server_stream.clone();
                        drop(stream);

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
                drop(stream);
                thread::sleep(Duration::from_secs(1));
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
    let topic_levels = publish.topic().levels();
    if topic_levels.len() == 1 && topic_levels[0] == NEW_INCIDENT {
        let message = String::from_utf8(publish.message().to_vec()).unwrap();
        handle_new_incident(message, drone, server_stream);
    }

    // if topic_levels.len() == 2 && topic_levels[0] == ATTENDING_INCIDENT {
    //     handle_attending_incident(publish.topic().to_string(), drone, server_stream);
    // } else if topic_levels.len() == 2 && topic_levels[0] == CLOSE_INCIDENT {
    //     handle_close_incident(topic_levels[1].bytes().to_string(), drone, server_stream);
    // }
}

fn handle_new_incident(
    message: String,
    drone: Arc<Mutex<Drone>>,
    server_stream: Arc<Mutex<TcpStream>>,
) {
    let incident = match Incident::from_string(message) {
        Ok(incident) => incident,
        Err(_) => {
            println!("Invalid incident message");
            return;
        }
    };

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

    drone_locked.set_incident(Some(incident.clone()));
    drop(drone_locked);

    println!(
        "Traveling to incident location: {}, {}",
        incident.x_coordinate, incident.y_coordinate
    );

    travel(drone.clone(), incident.x_coordinate, incident.y_coordinate);

    let mut drone_locked = match drone.lock() {
        Ok(drone) => drone,
        Err(_) => {
            println!("Mutex was poisoned");
            return;
        }
    };

    if drone_locked.status() != DroneStatus::Traveling {
        println!("Drone is not attending the incident");

        let x = drone_locked.x_default_coordinate();
        let y = drone_locked.y_default_coordinate();
        drone_locked.set_incident(None);

        drop(drone_locked);

        travel(drone.clone(), x, y);

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
    drop(drone_locked);

    let mut locked_stream = match server_stream.lock() {
        Ok(stream) => stream,
        Err(_) => {
            println!("Mutex was poisoned");
            return;
        }
    };

    let topic_filter = TopicFilter::new(
        vec![
            TopicLevel::Literal(ATTENDING_INCIDENT.to_vec()),
            TopicLevel::Literal(incident.uuid.clone().into_bytes()),
        ],
        false,
    );

    match unsubscribe(topic_filter, &mut locked_stream) {
        Ok(_) => println!("Unsubscribed from the incident topic"),
        Err(e) => eprintln!("Error: {:?}", e),
    }

    let topic_name = TopicName::new(
        vec![
            TopicLevel::Literal(ATTENDING_INCIDENT.to_vec()).to_bytes(),
            TopicLevel::Literal(incident.uuid.clone().into_bytes()).to_bytes(),
        ],
        false,
    );
    let message = b"".to_vec();

    match publish(topic_name, message, &mut locked_stream) {
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
    topic: String,
    drone: Arc<Mutex<Drone>>,
    server_stream: Arc<Mutex<TcpStream>>,
) {
    let incident_uuid = topic.split('/').last().unwrap().to_string();

    let mut drone = drone.lock().unwrap();

    drone.set_status(DroneStatus::AttendingIncident);
    println!("Drone is attending the incident");

    let close_topic = format!("close-incident/{}", incident_uuid);
    let topic_filter = TopicFilter::new(
        vec![TopicLevel::Literal(close_topic.as_bytes().to_vec())],
        false,
    );

    let mut stream = server_stream.lock().unwrap();
    subscribe(topic_filter, &mut stream).unwrap();
}

fn handle_close_incident(
    topic: String,
    drone: Arc<Mutex<Drone>>,
    server_stream: Arc<Mutex<TcpStream>>,
) {
    let incident_uuid = topic.split('/').last().unwrap().to_string();

    let mut drone = drone.lock().unwrap();

    println!("Drone returned to the central location");

    let close_topic = format!("close-incident/{}", incident_uuid);
    let topic_filter = TopicFilter::new(
        vec![TopicLevel::Literal(close_topic.as_bytes().to_vec())],
        false,
    );

    let mut stream = server_stream.lock().unwrap();
    unsubscribe(topic_filter, &mut stream).unwrap();
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

        let mut levels = vec![];
        levels.push(DRONE_DATA.to_vec());
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

        match publish(topic_name, message, &mut stream) {
            Ok(_) => println!("Drone data updated"),
            Err(e) => eprintln!("Error: {:?}", e),
        }

        // if drone.is_below_minimun() {
        //     println!("Drone battery is below minimum level");
        //     travel(drone, drone.central_x(), drone.central_y());
        // }

        drop(stream);

        thread::sleep(Duration::from_millis(UPDATE_INTERVAL));
    }
}

fn connect_to_server(address: &str) -> std::io::Result<TcpStream> {
    println!("\nConnecting to address: {:?}", address);
    let mut to_server_stream = TcpStream::connect(address)?;

    let client_id_bytes: Vec<u8> = b"drone".to_vec();
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
) -> std::io::Result<()> {
    let mut server_stream = server_stream.try_clone().unwrap();

    let dup = false;
    let qos = QoS::AtLeast;
    let retain = false;
    let package_identifier = Some(1);
    let message_bytes = message;

    let publish_packet = Publish::new(
        dup,
        qos,
        retain,
        topic_name,
        package_identifier,
        message_bytes,
    );

    let _ = server_stream.write(publish_packet.to_bytes().as_slice());

    server_stream.set_nonblocking(false).unwrap();
    match Packet::from_bytes(&mut server_stream) {
        Ok(Packet::Puback(_)) => Ok(()),
        _ => Err(std::io::Error::new(
            ErrorKind::Other,
            "Puback was not received.???",
        )),
    }
}

const DISCRETE_INTERVAL: f64 = 0.5;

fn travel(drone: Arc<Mutex<Drone>>, x: f64, y: f64) {
    let drone = drone.clone();

    println!("Traveling to ({}, {})", x, y);
    let thread = thread::spawn(move || {
        let mut locked_drone = drone.lock().unwrap();
        let mut distance = locked_drone.distance_to(x, y);
        let mut status = DroneStatus::Traveling;
        locked_drone.set_status(DroneStatus::Traveling);
        drop(locked_drone);

        while distance > 0.0 && status == DroneStatus::Traveling {
            println!("Distance to destination: {}", distance);
            let mut locked_drone = drone.lock().unwrap();

            locked_drone.travel_to(x, y);
            distance = locked_drone.distance_to(x, y);
            status = locked_drone.status();

            drop(locked_drone);
            thread::sleep(Duration::from_secs_f64(DISCRETE_INTERVAL));
        }
    });

    thread.join().unwrap();
}
