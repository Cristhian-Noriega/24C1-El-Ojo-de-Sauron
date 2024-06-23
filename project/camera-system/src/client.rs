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

const UPDATE_DATA_INTERVAL: u64 = 3;
const READ_MESSAGE_INTERVAL: u64 = 1;

/// Runs the client
pub fn client_run(config: Config) -> std::io::Result<()> {
    let key = config.get_key();
    let active_range = config.get_active_range();

    let mut server_stream = connect_to_server(&config.clone())?;
    let mut camera_system = CameraSystem::new();

    for (i, camera) in config.get_cameras().iter().enumerate() {
        let camara = Camera::new(
            i as u8,
            camera.x_coordinate.to_owned(),
            camera.y_coordinate.to_owned(),
            active_range,
        );
        camera_system.add_camera(camara);
    }

    make_initial_subscribes(&mut server_stream, key);

    let server_stream = Arc::new(Mutex::new(server_stream));
    let camera_system = Arc::new(Mutex::new(camera_system));

    let server_stream_clone = server_stream.clone();
    let camera_system_clone = camera_system.clone();

    let thread_update = thread::spawn(move || {
        update_camera_system_status(server_stream_clone, camera_system_clone, &key);
    });

    let server_stream_clone = server_stream.clone();
    let camera_system_clone = camera_system.clone();

    let thread_read = thread::spawn(move || {
        read_incoming_packets(server_stream_clone, camera_system_clone, &key);
    });

    thread_update.join().unwrap();
    thread_read.join().unwrap();
}

/// Read incoming packages in a loop
fn read_incoming_packets(
    server_stream: Arc<Mutex<TcpStream>>,
    camera_system: Arc<Mutex<CameraSystem>>,
    &key: &[u8; 32],
) {
    loop {
        let mut locked_stream = stream.lock().unwrap().try_clone().unwrap();
        locked_stream.set_nonblocking(true).unwrap();

        let incoming_publish = match Packet::from_bytes(&mut locked_stream, key) {
            Ok(Packet::Publish(publish)) => publish,
            _ => {
                drop(locked_stream);
                thread::sleep(Duration::from_secs(READ_MESSAGE_INTERVAL));
                continue;
            }
        };

        drop(locked_stream);

        let topic_levels = incoming_publish.topic().levels();

        if topic_levels.len() == 1 && topic_levels[0] == NEW_INCIDENT {
            handle_new_incident(incoming_publish, camera_system, key);
        } else if topic_levels.len() == 2 && topic_levels[0] == CLOSE_INCIDENT {
            handle_close_incident(incoming_publish, camera_system, key);
        }
    }
}

/// Periodically updates the camera system status
fn update_camera_system_status(
    server_stream: Arc<Mutex<TcpStream>>,
    camera_system: Arc<Mutex<CameraSystem>>,
    &key: &[u8; 32],
) {
    loop {
        let locked_camera_system = match camera_system.lock() {
            Ok(locked_camera_system) => locked_camera_system,
            Err(_) => {
                println!("Mutex was poisoned");
                return;
            }
        };

        let topic_name = TopicName::new(vec![CAMERA_DATA.to_vec()], false);
        let cameras_data = locked_camera_system.cameras_data().as_bytes().to_vec();

        drop(locked_camera_system);

        publish(topic_name, cameras_data, server_stream, key)?;

        thread::sleep(Duration::from_secs(UPDATE_DATA_INTERVAL));
    }
}

/// Publishes a message to a topic
fn publish(
    topic_name: TopicName,
    message: Vec<u8>,
    server_stream: Arc<Mutex<TcpStream>>,
    key: &[u8; 32],
) {
    let dup = false;
    let qos = QoS::AtMost;
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

    let mut locked_server_stream = match server_stream.lock() {
        Ok(locked_server_stream) => locked_server_stream,
        Err(_) => {
            println!("Mutex was poisoned");
            return;
        }
    };

    let _ = locked_server_stream.write(publish_packet.to_bytes(key).as_slice());

    drop(locked_server_stream);
}

/// Connects to the server
fn connect_to_server(config: &Config) -> std::io::Result<TcpStream> {
    let address = config.get_address().to_owned();
    let id = config.get_id().to_owned();
    let username = config.get_username().to_owned();
    let password = config.get_password().to_owned();
    let key = config.get_key().to_owned();

    println!("\nConnecting to address: {:?}", address);
    let mut to_server_stream = TcpStream::connect(address)?;

    let client_id_bytes: Vec<u8> = id.as_bytes().to_vec();
    let client_id = EncodedString::new(client_id_bytes);
    let will = None;

    let username = EncodedString::new(username.as_bytes().to_vec());
    let password = Some(EncodedString::new(password.as_bytes().to_vec()));
    let login = Some(Login::new(username, password));

    let connect = Connect::new(false, 0, client_id, will, login);

    let _ = to_server_stream.write(connect.to_bytes(&key).as_slice());

    match Packet::from_bytes(&mut to_server_stream, &key) {
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

/// Handles a new incident
fn handle_new_incident(
    incoming_publish: Publish,
    camera_system: Arc<Mutex<CameraSystem>>,
    key: &[u8; 32],
) {
    let incident_string = String::from_utf8_lossy(incoming_publish.message()).to_string();
    let incident = match Incident::from_string(incident_string) {
        Ok(incident) => incident,
        Err(_) => {
            println!("Malformated incident");
            return;
        }
    };

    let locked_camera_system = match camera_system.lock() {
        Ok(locked_camera_system) => locked_camera_system,
        Err(_) => {
            println!("Mutex was poisoned");
            return;
        }
    };

    locked_camera_system.new_incident(incident.clone());

    drop(locked_camera_system);
}

/// Handles the closing of an incident
fn handle_close_incident(
    incoming_publish: Publish,
    camera_system: Arc<Mutex<CameraSystem>>,
    key: &[u8; 32],
) {
    let topic_levels = incoming_publish.topic().levels();
    let incident_id = String::from_utf8_lossy(topic_levels[1].as_slice()).to_string();

    let locked_camera_system = match camera_system.lock() {
        Ok(locked_camera_system) => locked_camera_system,
        Err(_) => {
            println!("Mutex was poisoned");
            return;
        }
    };

    locked_camera_system.close_incident(&incident_id);

    drop(locked_camera_system);
}

/// Make initial subscribes
fn make_initial_subscribes(server_stream: &mut TcpStream, key: &[u8; 32]) {
    let new_incident = TopicFilter::new(vec![TopicLevel::Literal(NEW_INCIDENT.to_vec())], false);
    let close_incident = TopicFilter::new(
        vec![
            TopicLevel::Literal(CLOSE_INCIDENT.to_vec()),
            TopicLevel::SingleLevelWildcard,
        ],
        false,
    );
    let topics = vec![new_incident, close_incident];
    subscribe(topics, &mut server_stream, key);
}

/// Handles the subscription to a topic
fn subscribe(filter: Vec<TopicFilter>, server_stream: &mut TcpStream, key: &[u8; 32]) {
    let mut topics_filters = vec![];

    for topic_filter in filter {
        topics_filters.push((topic_filter, QoS::AtLeast));
    }

    let packet_id = 1;

    let subscribe_packet = Subscribe::new(packet_id, topics_filters);
    let _ = server_stream.write(subscribe_packet.to_bytes(key).as_slice());

    match Packet::from_bytes(server_stream, key) {
        Ok(Packet::Suback(suback)) => {}
        _ => {
            println!("Suback was not recibed");
            return;
        }
    }
}
