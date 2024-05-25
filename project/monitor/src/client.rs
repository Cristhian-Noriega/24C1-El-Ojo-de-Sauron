use std::env::args;
use std::io::stdin;
use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;
use sauron::model::encoded_string::EncodedString;
use std::thread;
use sauron::model::components::encoded_string::EncodedString;
use sauron::model::components::topic_name::TopicName;
pub use sauron::model::{
    components::{qos::QoS, topic_filter::TopicFilter, topic_level::TopicLevel},
    packet::Packet,
    packets::{
        connect::Connect, disconnect::Disconnect, pingreq::Pingreq, puback::Puback,
        publish::Publish, subscribe::Subscribe, unsubscribe::Unsubscribe,
    },
};

static CLIENT_ARGS: usize = 3;

pub struct Client {
    pub connection_status: String,
    pub response_text: String,
    pub response_bytes: String,
    pub address: String,
}

impl Client {
    pub fn new() -> Self {
        let argv = args().collect::<Vec<String>>();
        if argv.len() != CLIENT_ARGS {
            let app_name = &argv[0];
            println!("{:?} <host> <puerto>", app_name);
        }

        let address = argv[1].clone() + ":" + &argv[2];

        Self {
            connection_status: "offline".to_owned(),
            response_text: "no response".to_owned(),
            response_bytes: "no response".to_owned(),
            address: address,
        }
    }

    pub fn send_connect(&mut self) -> std::io::Result<()> {
        println!("Connecting to {}", self.address);

        let from_server_stream = &mut stdin();

        let mut to_server_stream = TcpStream::connect(self.address)?;
        let reader = BufReader::new(from_server_stream);

        let client_id_bytes: Vec<u8> = b"monitor".to_vec();
        let client_id = EncodedString::new(client_id_bytes);
        let will = None;
        let login = None;
        let connect_package = Connect::new(false, 0, client_id, will, login);

        let mut to_server_stream_clone = to_server_stream.try_clone()?;
        thread::spawn(move || {
            loop {
                let mut buffer = [0; 1024];
                let _ = to_server_stream_clone.read(&mut buffer);
                let packet = Packet::from_bytes(&mut buffer.as_slice()).unwrap();

                match packet {
                    Packet::Connack(connack) => {
                        println!(
                            "Received Connack packet with return code: {:?} and sessionPresent: {:?}",
                            connack.connect_return_code(),
                            connack.session_present()
                        );
                    }
                    Packet::Publish(publish) => {
                        println!("Received Publish packet {:?}", publish);

                        let message = publish.message();
                        let message_str = String::from_utf8_lossy(message).to_string();

                        println!("Message: {:?}", message_str);
                    }
                    
                    _ => println!("Received unsupported packet type"),
                }
            }
        });

        self.connection_status = "connected".to_owned();
        self.response_text = "CONNACK".to_owned();
        self.response_bytes = "CONNACK".to_owned();
    }
}