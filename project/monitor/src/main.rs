#![allow(unused_variables)]
use std::env::args;
use std::io::stdin;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;

use sauron::model::components::encoded_string::EncodedString;
pub use sauron::model::{
    components::{qos::QoS, topic_level::TopicLevel, topic_name::TopicName},
    packet::Packet,
    packets::{
        connect::Connect, disconnect::Disconnect, pingresp::Pingresp, puback::Puback,
        publish::Publish, subscribe::Subscribe, unsubscribe::Unsubscribe,
    },
};

static CLIENT_ARGS: usize = 3;

fn main() -> Result<(), ()> {
    let argv = args().collect::<Vec<String>>();
    if argv.len() != CLIENT_ARGS {
        println!("Cantidad de argumentos inválido");
        let app_name = &argv[0];
        println!("{:?} <host> <puerto>", app_name);
        return Err(());
    }

    let address = argv[1].clone() + ":" + &argv[2];
    println!("Conectándome a {:?}", address);

    match client_run(&address, &mut stdin()) {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("Error: {:?}", e);
            Err(())
        }
    }
}

fn client_run(address: &str, from_server_stream: &mut dyn Read) -> std::io::Result<()> {
    let mut to_server_stream = TcpStream::connect(address)?;

    //client id: monitor app
    let client_id_bytes = b"monitor app".to_vec();
    let client_id = EncodedString::new(client_id_bytes);

    let will = None;
    let login = None;
    let connect_package = Connect::new(false, 0, client_id, will, login);

    let _ = to_server_stream.write(connect_package.to_bytes().as_slice());

    // Read the Connack packet from the server
    let packet = Packet::from_bytes(&mut to_server_stream).unwrap();

    match packet {
        Packet::Connack(connack) => {
            println!(
                "Received Connack packet with return code: {:?} and sessionPresent: {:?}",
                connack.connect_return_code(),
                connack.session_present()
            );
        }
        _ => {
            println!("Received unsupported packet type");
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Received unsupported packet type",
            ));
        }
    }

    // Subscribe to the topic "test"

    println!("Enter the topic to publish to:");
    let mut topic = String::new();
    std::io::stdin().read_line(&mut topic)?;

    println!("Enter the message to publish:");
    let mut message = String::new();
    std::io::stdin().read_line(&mut message)?;

    let mut levels = vec![];

    for level in topic.split(' ') {
        if let Ok(TopicLevel::Literal(literal)) = TopicLevel::from_bytes(level.as_bytes().to_vec()) {
            levels.push(literal);
        }
    }

    let dup = false;
    let qos = QoS::AtMost;
    let retain = false;
    let topic_name = TopicName::new(levels, false);
    let package_identifier = None;
    let message_bytes = message.as_bytes().to_vec();

    let publish_packet = Publish::new(
        dup,
        qos,
        retain,
        topic_name,
        package_identifier,
        message_bytes,
    );

    let _ = to_server_stream.write(publish_packet.to_bytes().as_slice());

    // Read the Puback packet from the server

    let packet = Packet::from_bytes(&mut to_server_stream).unwrap();

    match packet {
        Packet::Puback(puback) => {
            println!(
                "Received Puback packet with package identifier: {:?}",
                puback
            );
        }
        _ => {
            println!("Received unsupported packet type");
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Received unsupported packet type",
            ));
        }
    }

    // Loop to read packets from the server

    loop {
        let packet = Packet::from_bytes(&mut to_server_stream).unwrap();

        match packet {
            Packet::Publish(publish) => {
                println!("Received Publish packet {:?}", publish);

                let message = publish.message();
                let message_str = String::from_utf8_lossy(message).to_string();

                println!("Message: {:?}", message_str);
            }
            _ => {
                println!("Received unsupported packet type");
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Received unsupported packet type",
                ));
            }
        }
    }
}
