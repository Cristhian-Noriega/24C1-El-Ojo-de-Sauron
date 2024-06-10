#![allow(clippy::unused_io_amount)]

use mqtt::model::{
    components::encoded_string::EncodedString, components::qos::QoS,
    components::topic_filter::TopicFilter, components::topic_level::TopicLevel,
    components::topic_name::TopicName, packet::Packet, packets::connect::Connect,
    packets::publish::Publish, packets::subscribe::Subscribe,
};
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::{env::args, thread};

use crate::monitor::Monitor;

static CLIENT_ARGS: usize = 3;

const CAMERA_DATA: &[u8] = b"camera-data";
const DRON_DATA: &[u8] = b"dron-data";
const ATTEND_INCIDENT: &[u8] = b"attend-incident";
const CLOSE_INCIDENT: &[u8] = b"close-incident";

pub struct Client {
    pub connection_status: Arc<Mutex<String>>,
    pub address: String,
    pub to_server_stream: Arc<Mutex<Option<TcpStream>>>,
    pub sender: Sender<String>,
}

impl Client {
    pub fn new(sender: Sender<String>) -> Self {
        let argv = args().collect::<Vec<String>>();
        if argv.len() != CLIENT_ARGS {
            let app_name = &argv[0];
            println!("{:?} <host> <puerto>", app_name);
        }

        let address = argv[1].clone() + ":" + &argv[2];

        Self {
            connection_status: Arc::new(Mutex::new("disconnected".to_owned())),
            address,
            to_server_stream: Arc::new(Mutex::new(None)),
            sender,
        }
    }

    pub fn connect_to_server(&self) -> std::io::Result<TcpStream> {
        println!("\nConnecting to address: {:?}", self.address);
        let mut to_server_stream = TcpStream::connect(&self.address)?;

        let client_id_bytes: Vec<u8> = b"monitor".to_vec();
        let client_id = EncodedString::new(client_id_bytes);
        let will = None;
        let login = None;
        let connect_package = Connect::new(false, 0, client_id, will, login);

        let _ = to_server_stream.write(connect_package.to_bytes().as_slice());
        // update the field to_server_stream
        *self.to_server_stream.lock().unwrap() = Some(to_server_stream.try_clone()?);

        match Packet::from_bytes(&mut to_server_stream) {
            Ok(Packet::Connack(connack)) => {
                println!(
                    "Received Connack packet with return code: {:?} and sessionPresent: {:?}\n",
                    connack.connect_return_code(),
                    connack.session_present()
                );
                let connection_status = Arc::clone(&self.connection_status);
                "connected".clone_into(&mut connection_status.lock().unwrap());
                Ok(to_server_stream)
            }
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Connack was not received.",
            )),
        }
    }

    pub fn client_run(&mut self, monitor: &Monitor) -> std::io::Result<()> {
        let to_server_stream = self.connect_to_server()?;

        self.make_initial_subscribes()?;

        let mut to_server_stream_clone = to_server_stream.try_clone()?;
        let sender = self.sender.clone();
        let thread_packet = thread::spawn(move || {
            loop {
                let mut buffer = [0; 1024];
                match to_server_stream_clone.read(&mut buffer) {
                    Ok(_) => {
                        let packet = Packet::from_bytes(&mut buffer.as_slice()).unwrap();
                        match packet {
                            // Packet::Connack(connack) => {
                            //     println!(
                            //         "Received Connack packet with return code: {:?} and sessionPresent: {:?}",
                            //         connack.connect_return_code(),
                            //         connack.session_present()
                            //     );
                            //     "connected".clone_into(&mut connection_status.lock().unwrap());
                            //     *response_text.lock().unwrap() = format!("{}", connack);
                            // }
                            Packet::Publish(publish) => {
                                let topic_levels = publish.topic().levels();

                                if topic_levels.len() == 2 && topic_levels[0] == CAMERA_DATA {
                                    println!("Camera data received\n");
                                    //monitor.handle_camera_data(publish);
                                } else if topic_levels.len() == 2 && topic_levels[0] == DRON_DATA {
                                    println!("Dron data received\n");
                                    //monitor.handle_dron_data(publish);
                                } else if topic_levels.len() == 2
                                    && topic_levels[0] == ATTEND_INCIDENT
                                {
                                    println!("Attend Incident received\n");
                                    //monitor.handle_attend_incident_data(publish);
                                } else if topic_levels.len() == 2
                                    && topic_levels[0] == CLOSE_INCIDENT
                                {
                                    println!("Close Incident received\n");
                                    //monitor.handle_close_incident_data(publish);
                                } else {
                                    println!("Unknown topic received\n");
                                }
                            }
                            Packet::Puback(puback) => {
                                println!("Received Puback packet!\n");
                                // *response_text.lock().unwrap() = format!("{:?}", puback);
                                let puback_info = format!("{}", puback);
                                // Update the response_text field with the Puback packet information
                                sender.send(format!("{}", puback)).unwrap();
                            }
                            Packet::Pingresp(_pingresp) => {
                                println!("Received Pingresp packet\n");
                            }
                            Packet::Suback(_suback) => {
                                println!("Received Suback packet\n");
                            }
                            Packet::Unsuback(_unsuback) => {
                                println!("Received Unsuback packet\n");
                            }
                            Packet::Pingreq(_pingreq) => {
                                println!("Received Pingreq packet\n");
                            }
                            Packet::Disconnect(_disconnect) => {
                                println!("Received Disconnect packet\n");
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
        });

        thread_packet.join().unwrap();

        Ok(())
    }

    pub fn publish(&self, topic: &str, message: &str) -> std::io::Result<()> {
        let mut levels = vec![];
        let message = message.trim();

        for level in topic.split(' ') {
            if let Ok(TopicLevel::Literal(literal)) =
                TopicLevel::from_bytes(level.as_bytes().to_vec())
            {
                levels.push(literal);
            }
        }

        let dup = false;
        let qos = QoS::AtLeast;
        let retain = false;
        let package_identifier = Some(1);
        let topic_name = TopicName::new(levels, false);
        let message_bytes = message.as_bytes().to_vec();

        let publish_packet = Publish::new(
            dup,
            qos,
            retain,
            topic_name,
            package_identifier,
            message_bytes,
        );

        //println!("Packet Publish: {:?}", publish_packet);
        let _ = self
            .to_server_stream
            .lock()
            .unwrap()
            .as_mut()
            .unwrap()
            .write(publish_packet.to_bytes().as_slice());
        println!(
            "Sent Publish packet to topic: {:?} with message: {:?}",
            topic, message
        );

        Ok(())
    }

    pub fn subscribe(&self, topics: Vec<&str>) -> std::io::Result<()> {
        let mut topics_filters = vec![];

        for topic in topics {
            let mut levels = vec![];
            for level in topic.split('/') {
                if let Ok(topic_level) = TopicLevel::from_bytes(level.as_bytes().to_vec()) {
                    levels.push(topic_level);
                }
            }

            let topic_filter = TopicFilter::new(levels, false);
            let qos = QoS::AtLeast;

            topics_filters.push((topic_filter, qos));
        }

        let packet_id = 1;
        let subscribe_packet = Subscribe::new(packet_id, topics_filters);

        println!("Packet ID: {:?}", subscribe_packet.packet_identifier());
        let _ = self
            .to_server_stream
            .lock()
            .unwrap()
            .as_mut()
            .unwrap()
            .write(subscribe_packet.to_bytes().as_slice());
        println!("Sent Subscribe packet");

        match Packet::from_bytes(self.to_server_stream.lock().unwrap().as_mut().unwrap()) {
            Ok(Packet::Suback(_)) => Ok(()),
            _ => Err(std::io::Error::new(
                ErrorKind::Other,
                "Suback was not received.",
            )),
        }
    }

    fn make_initial_subscribes(&self) -> std::io::Result<()> {
        let topics = vec![
            "camera-data",
            "camera-update",
            "attending-incident/+",
            "close-incident/+",
        ];

        self.subscribe(topics)
    }
}
