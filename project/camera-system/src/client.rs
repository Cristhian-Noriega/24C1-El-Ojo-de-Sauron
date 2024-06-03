use std::io::ErrorKind;
use std::io::Write;
use std::net::TcpStream;

pub use mqtt::model::{
    components::{qos::QoS, topic_filter::TopicFilter, topic_level::TopicLevel},
    packet::Packet,
    packets::{connect::Connect, publish::Publish, subscribe::Subscribe, unsubscribe::Unsubscribe},
};

use mqtt::model::components::encoded_string::EncodedString;
use mqtt::model::components::topic_name::TopicName;

use crate::camera::Camera;
use crate::camera_system::CameraSystem;
use crate::incident::Incident;

const CAMERA_QUANTITY: usize = 3;

const NEW_INCIDENT: &[u8] = b"new-incident";
const CLOSE_INCIDENT: &[u8] = b"close-incident";
const CAMERA_DATA: &[u8] = b"camera-data";

pub fn client_run(address: &str) -> std::io::Result<()> {
    let mut server_stream = connect_to_server(address)?;

    let mut camera_system = CameraSystem::new();

    for i in 0..CAMERA_QUANTITY {
        let camara = Camera::new(i as u8, i as f64, i as f64);
        camera_system.add_camera(camara);
    }

    publish_camera_state(&mut camera_system, &mut server_stream)?;

    let topic_filter = TopicFilter::new(vec![TopicLevel::Literal(NEW_INCIDENT.to_vec())], false);

    subscribe(topic_filter, &mut server_stream)?;

    loop {
        let incoming_publish = match Packet::from_bytes(&mut server_stream) {
            Ok(Packet::Publish(publish)) => publish,
            _ => {
                return Err(std::io::Error::new(
                    ErrorKind::Other,
                    "Unexpected packet type received",
                ))
            }
        };

        let topic_levels = incoming_publish.topic().levels();

        if topic_levels.len() == 1 && topic_levels[0] == NEW_INCIDENT {
            handle_new_incident(incoming_publish, &mut camera_system, &mut server_stream)?;
        } else if topic_levels.len() == 2 && topic_levels[0] == CLOSE_INCIDENT {
            handle_close_incident(incoming_publish, &mut camera_system, &mut server_stream)?;
        }
    }
}

fn unsubscribe(filter: TopicFilter, server_stream: &mut TcpStream) -> std::io::Result<()> {
    let packet_id = 1;

    let topics_filters = vec![(filter)];

    let unsubscribe_packet = Unsubscribe::new(packet_id, topics_filters);

    let _ = server_stream.write(unsubscribe_packet.to_bytes().as_slice());

    match Packet::from_bytes(server_stream) {
        Ok(Packet::Unsuback(_)) => Ok(()),
        _ => Err(std::io::Error::new(
            ErrorKind::Other,
            "Unsuback was not received.",
        )),
    }
}

fn subscribe(filter: TopicFilter, server_stream: &mut TcpStream) -> std::io::Result<()> {
    let packet_id = 1;
    let qos = QoS::AtLeast;

    let topics_filters = vec![(filter, qos)];

    let subscribe_packet = Subscribe::new(packet_id, topics_filters);

    let _ = server_stream.write(subscribe_packet.to_bytes().as_slice());

    match Packet::from_bytes(server_stream) {
        Ok(Packet::Suback(_)) => Ok(()),
        _ => Err(std::io::Error::new(
            ErrorKind::Other,
            "Suback was not received.",
        )),
    }
}

fn publish(
    topic_name: TopicName,
    message: Vec<u8>,
    server_stream: &mut TcpStream,
) -> std::io::Result<()> {
    let dup = false;
    let qos = QoS::AtMost;
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

    match Packet::from_bytes(server_stream) {
        Ok(Packet::Puback(_)) => Ok(()),
        _ => Err(std::io::Error::new(
            ErrorKind::Other,
            "Puback was not received.",
        )),
    }
}

fn connect_to_server(address: &str) -> std::io::Result<TcpStream> {
    println!("\nConnecting to address: {:?}", address);
    let mut to_server_stream = TcpStream::connect(address)?;

    let client_id_bytes: Vec<u8> = b"camera system".to_vec();
    let client_id = EncodedString::new(client_id_bytes);
    let will = None;
    let login = None;
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

fn publish_camera_state(
    camera_system: &mut CameraSystem,
    server_stream: &mut TcpStream,
) -> std::io::Result<()> {
    let cameras_data = camera_system.cameras_data().as_bytes().to_vec();
    let topic_name = TopicName::new(vec![CAMERA_DATA.to_vec()], false);

    publish(topic_name, cameras_data, server_stream)
}

// new-incident
//      activar las camaras correspondientes
//      publicar el estado actualizado de la camaras
//      subscribe to close-incident/uuid

fn handle_new_incident(
    incoming_publish: Publish,
    camera_system: &mut CameraSystem,
    server_stream: &mut TcpStream,
) -> std::io::Result<()> {
    let incident_string = String::from_utf8_lossy(incoming_publish.message()).to_string();
    let incident = match Incident::from_string(incident_string) {
        Ok(incident) => incident,
        Err(_) => {
            return Err(std::io::Error::new(
                ErrorKind::Other,
                "Error parsing incident",
            ))
        }
    };

    camera_system.new_incident(incident.clone());

    publish_camera_state(camera_system, server_stream)?;

    let incident_id = incident.uuid();

    let topic_levels = vec![
        TopicLevel::Literal(CLOSE_INCIDENT.to_vec()),
        TopicLevel::Literal(incident_id.as_bytes().to_vec()),
    ];
    let topic_filter = TopicFilter::new(topic_levels, false);

    subscribe(topic_filter, server_stream)?;
    Ok(())
}

// close-incident/uuid
//      pasar a Sleep las camaras correspondientes
//      publicar el estado actualizado de las camaras
//      unsubscribe from close-incident/uuid
fn handle_close_incident(
    incoming_publish: Publish,
    camera_system: &mut CameraSystem,
    server_stream: &mut TcpStream,
) -> std::io::Result<()> {
    let topic_levels = incoming_publish.topic().levels();

    let incident_id = String::from_utf8_lossy(topic_levels[1].as_slice()).to_string();
    camera_system.close_incident(&incident_id);

    publish_camera_state(camera_system, server_stream)?;

    let topic_levels = vec![
        TopicLevel::Literal(CLOSE_INCIDENT.to_vec()),
        TopicLevel::Literal(topic_levels[1].to_vec()),
    ];
    let topic_filter = TopicFilter::new(topic_levels, false);

    unsubscribe(topic_filter, server_stream)?;
    Ok(())
}
