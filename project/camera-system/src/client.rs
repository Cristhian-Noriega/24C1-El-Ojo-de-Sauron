use std::{
    io::{ErrorKind, Write},
    net::TcpStream,
};

use common::incident::Incident;
use mqtt::model::{
    components::{
        encoded_string::EncodedString, login::Login, qos::QoS, topic_filter::TopicFilter,
        topic_level::TopicLevel, topic_name::TopicName,
    },
    packet::Packet,
    packets::{connect::Connect, publish::Publish, subscribe::Subscribe},
    return_codes::connect_return_code::ConnectReturnCode,
};

use crate::camera_system::CameraSystem;

use crate::{camera::Camera, config::Config};

const NEW_INCIDENT: &[u8] = b"new-incident";
const CLOSE_INCIDENT: &[u8] = b"close-incident";
const CAMERA_DATA: &[u8] = b"camera-data";

pub fn client_run(config: Config) -> std::io::Result<()> {
    let address = config.get_address().to_owned();
    let username = config.get_username().to_owned();
    let password = config.get_password().to_owned();

    let mut server_stream = connect_to_server(&address, &username, &password)?;

    let mut camera_system = CameraSystem::new();

    // TODO: camaras reales
    for (i, camera) in config.get_cameras().iter().enumerate() {
        let camara = Camera::new(
            i as u8,
            camera.x_coordinate.to_owned(),
            camera.y_coordinate.to_owned(),
        );
        camera_system.add_camera(camara);
    }

    publish_camera_state(&mut camera_system, &mut server_stream)?;

    let new_incident = TopicFilter::new(vec![TopicLevel::Literal(NEW_INCIDENT.to_vec())], false);
    let close_incident = TopicFilter::new(
        vec![
            TopicLevel::Literal(CLOSE_INCIDENT.to_vec()),
            TopicLevel::SingleLevelWildcard,
        ],
        false,
    );
    let topics = vec![new_incident, close_incident];
    subscribe(topics, &mut server_stream)?;

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

// fn unsubscribe(filter: TopicFilter, server_stream: &mut TcpStream) -> std::io::Result<()> {
//     let packet_id = 1;

//     let topics_filters = vec![(filter)];

//     let unsubscribe_packet = Unsubscribe::new(packet_id, topics_filters);

//     let _ = server_stream.write(unsubscribe_packet.to_bytes().as_slice());

//     match Packet::from_bytes(server_stream) {
//         Ok(Packet::Unsuback(_)) => Ok(()),
//         _ => Err(std::io::Error::new(
//             ErrorKind::Other,
//             "Unsuback was not received.",
//         )),
//     }
// }

fn subscribe(filter: Vec<TopicFilter>, server_stream: &mut TcpStream) -> std::io::Result<()> {
    let mut topics_filters = vec![];

    for topic_filter in filter {
        topics_filters.push((topic_filter, QoS::AtLeast));
    }

    let packet_id = 1;

    // let topics_filters = vec![(filter, qos)];

    let subscribe_packet = Subscribe::new(packet_id, topics_filters);
    println!(
        "Subscribe packet: {:?}",
        subscribe_packet.to_bytes().as_slice()
    );
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

    match Packet::from_bytes(server_stream) {
        Ok(Packet::Puback(_)) => Ok(()),
        _ => Err(std::io::Error::new(
            ErrorKind::Other,
            "Puback was not received.",
        )),
    }
}

fn connect_to_server(address: &str, username: &str, password: &str) -> std::io::Result<TcpStream> {
    println!("\nConnecting to address: {:?}", address);
    let mut to_server_stream = TcpStream::connect(address)?;

    let client_id_bytes: Vec<u8> = b"camera system".to_vec();
    let client_id = EncodedString::new(client_id_bytes);
    let will = None;

    let username = EncodedString::new(username.as_bytes().to_vec());
    let password = Some(EncodedString::new(password.as_bytes().to_vec()));
    let login = Some(Login::new(username, password));

    let connect = Connect::new(false, 0, client_id, will, login);

    let _ = to_server_stream.write(connect.to_bytes().as_slice());

    match Packet::from_bytes(&mut to_server_stream) {
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

    println!("New incident: {:?}", incident);

    camera_system.new_incident(incident.clone());

    publish_camera_state(camera_system, server_stream)?;
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

    Ok(())
}
