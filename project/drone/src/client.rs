use std::{
    io::{ErrorKind, Read, Write},
    net::TcpStream,
    sync::{Arc, Mutex, MutexGuard},
    thread,
};

use mqtt::model::{
    components::{
        encoded_string::EncodedString, qos::QoS, topic_filter::TopicFilter,
        topic_level::TopicLevel, topic_name::TopicName,
    },
    packet::Packet,
    packets::{connect::Connect, publish::Publish, subscribe::Subscribe, unsubscribe::Unsubscribe},
};

use crate::{drone::Drone, drone_status::DroneStatus};

const NEW_INCIDENT: &[u8] = b"new-incident";
const ATTENDING_INCIDENT: &[u8] = b"attending-incident";
const CLOSE_INCIDENT: &[u8] = b"close-incident";
const DRONE_DATA: &[u8] = b"drone-data";

const UPDATE_INTERVAL: u64 = 5;

pub fn client_run(address: &str) -> std::io::Result<()> {
    let server_stream = connect_to_server(address)?;
    let server_stream = Arc::new(Mutex::new(server_stream));

    let drone = Arc::new(Mutex::new(Drone::new()));

    let new_incident = TopicFilter::new(vec![TopicLevel::Literal(NEW_INCIDENT.to_vec())], false);

    let server_stream_clone = server_stream.clone();
    let drone_clone = drone.clone();

    thread::spawn(move || {
        read_incoming_packets(server_stream_clone, drone_clone);
    });

    match server_stream.lock() {
        Ok(mut server_stream) => {
            subscribe(new_incident, &mut server_stream)?;
        }
        Err(_) => {
            return Err(std::io::Error::new(ErrorKind::Other, "Mutex was poisoned"));
        }
    }

    update_drone_status(server_stream, drone.clone());

    Ok(())
}


fn read_incoming_packets(server_stream: Arc<Mutex<TcpStream>>, drone: Arc<Mutex<Drone>>){
    loop {
        let mut buffer = [0; 1024];
        let mut stream = server_stream.lock().unwrap();

        match stream.read(&mut buffer) {
            Ok(_) => {
                let packet = Packet::from_bytes(&mut buffer.as_slice()).unwrap();
                match packet {
                    Packet::Publish(publish) => {
                        handle_publish(publish, drone.clone(), server_stream.clone());
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
            Err(e) => {
                println!("Lost connection to server: {:?}", e);
                break;
            }
        }
    }
}

fn handle_publish(publish: Publish, drone: Arc<Mutex<Drone>>, server_stream: Arc<Mutex<TcpStream>>) {
    todo!();
}   


fn handle_new_incident(message: String, drone: Arc<Mutex<Drone>>, server_stream: Arc<Mutex<TcpStream>>) {
    let incident_data: Vec<&str> = message.split(';').collect();
    let x_incident = incident_data[0].parse::<f64>().unwrap();
    let y_incident = incident_data[1].parse::<f64>().unwrap();
    let incident_uuid = incident_data[2].as_bytes().to_vec();

    let mut drone = match drone.lock() {
        Ok(drone) => drone,
        Err(_) => {
            println!("Mutex was poisoned");
            return;
        }
    };

    if !drone.is_below_minimun() && drone.is_within_range(x_incident, y_incident) {
        drone.set_position(x_incident, y_incident);
        println!("Drone moved to the new incident location: ({}, {})", x_incident, y_incident);
        //add the uuid of the incident to the attending incidents topic
        let topic_filter = TopicFilter::new(vec![TopicLevel::Literal(ATTENDING_INCIDENT.to_vec()), TopicLevel::Literal(incident_uuid)], false);

        let mut stream = server_stream.lock().unwrap();
        subscribe(topic_filter, &mut stream).unwrap();
    } 
    
}

fn handle_attending_incident(topic: String, drone: Arc<Mutex<Drone>>, server_stream: Arc<Mutex<TcpStream>>) {
    let incident_uuid = topic.split('/').last().unwrap().to_string();

    let mut drone = drone.lock().unwrap();

    drone.set_state(DroneStatus::AttendingIncident);
    println!("Drone is attending the incident");

    let close_topic = format!("close-incident/{}", incident_uuid);
    let topic_filter = TopicFilter::new(vec![TopicLevel::Literal(close_topic.as_bytes().to_vec())], false);

    let mut stream = server_stream.lock().unwrap();
    subscribe(topic_filter, &mut stream).unwrap();
}

fn handle_close_incident(topic: String, drone: Arc<Mutex<Drone>>, server_stream: Arc<Mutex<TcpStream>>) {
    let incident_uuid = topic.split('/').last().unwrap().to_string();

    let mut drone = drone.lock().unwrap();

    drone.return_to_central();
    println!("Drone returned to the central location");

    let close_topic = format!("close-incident/{}", incident_uuid);
    let topic_filter = TopicFilter::new(vec![TopicLevel::Literal(close_topic.as_bytes().to_vec())], false);

    let mut stream = server_stream.lock().unwrap();
    unsubscribe(topic_filter, &mut stream).unwrap();
}


fn update_drone_status(server_stream: Arc<Mutex<TcpStream>>, drone: Arc<Mutex<Drone>>) {
    thread::spawn(move || loop {
        let mut stream = match server_stream.lock() {
            Ok(server_stream) => server_stream,
            Err(_) => {
                println!("Mutex was poisoned");
                return;
            }
        };

        let mut drone = match drone.lock() {
            Ok(drone) => drone,
            Err(_) => {
                println!("Mutex was poisoned");
                return;
            }
        };

        drone.update_state();

        match publish_drone_state(&drone, &mut stream) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error: {:?}", e);
                return;
            }
        }

        if drone.is_below_minimun() {
            println!("Drone battery is below minimum level");
            drone.return_to_central();
        }

        thread::sleep(std::time::Duration::from_secs(UPDATE_INTERVAL));
    });
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

fn publish_drone_state(
    drone: &MutexGuard<Drone>,
    server_stream: &mut MutexGuard<TcpStream>,
) -> std::io::Result<()> {
    let mut levels = vec![];
    levels.push(DRONE_DATA.to_vec());
    levels.push(drone.id().to_string().into_bytes());

    let topic_name = TopicName::new(levels, false);
    let message = drone.data().into_bytes();

    publish(topic_name, message, server_stream)?;
    Ok(())
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
    println!(
        "Subscribe packet: {:?}",
        subscribe_packet.to_bytes().as_slice()
    );
    let _ = server_stream.write(subscribe_packet.to_bytes().as_slice());

    match Packet::from_bytes(&mut server_stream) {
        Ok(Packet::Suback(_)) => Ok(()),
        _ => Err(std::io::Error::new(ErrorKind::Other,"Suback was not received.",
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
    let package_identifier = None;
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

    match Packet::from_bytes(&mut server_stream) {
        Ok(Packet::Puback(_)) => Ok(()),
        _ => Err(std::io::Error::new(
            ErrorKind::Other,
            "Puback was not received.",
        )),
    }
}
