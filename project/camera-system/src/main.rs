use std::env::args;
use std::io::stdin;
use std::io::Write;
use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;

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

//the client receives a connack packet from the server
fn client_run(address: &str, from_server_stream: &mut dyn Read) -> std::io::Result<()> {
    let mut to_server_stream = TcpStream::connect(address)?;
    let reader = BufReader::new(from_server_stream);

    //client id: camera system
    let client_id_bytes: Vec<u8> = b"camera system".to_vec();
    let client_id = EncodedString::new(client_id_bytes);
    let will = None;
    let login = None;
    let connect_package = Connect::new(false, 0, client_id, will, login);

    let _ = to_server_stream.write(connect_package.to_bytes().as_slice());

    // Read the Connack packet from the server
    let mut buffer = [0; 1024];
    let _ = to_server_stream.read(&mut buffer);
    let packet = Packet::from_bytes(&mut buffer.as_slice()).unwrap();

    match packet {
        Packet::Connack(connack) => {
            println!(
                "Received Connack packet with return code: {:?} and sessionPresent: {:?}",
                connack.connect_return_code(),
                connack.session_present()
            );
        }
        _ => println!("Received unsupported packet type"),
    }

    for line in reader.lines().map_while(Result::ok) {
        let command = line.trim();
        if command == "subscribe" {
            println!("Enter the topic to subscribe to (sepataded by spaces):");
            let mut topic = String::new();
            std::io::stdin().read_line(&mut topic)?;

            let mut levels = vec![];
            for level in topic.split(' ') {
                if let Ok(topic_level) = TopicLevel::from_bytes(level.as_bytes().to_vec()) {
                    levels.push(topic_level);
                }
            }

            let topic_filter = TopicFilter::new(levels, false);
            let packet_id = 1;
            let qos = QoS::AtLeast;

            let topics_filters = vec![(topic_filter, qos)];

            let subscribe_packet = Subscribe::new(packet_id, topics_filters);

            // Send Subscribe packet
            println!("Packet ID: {:?}", subscribe_packet.packet_identifier);
            println!("Topics: {:?}", subscribe_packet.topics);
            let _ = to_server_stream.write(subscribe_packet.to_bytes().as_slice());
            println!("Sent Subscribe packet");
        }
        if command == "publish" {
            println!("Enter the topic to publish to:");
            let mut topic = String::new();
            std::io::stdin().read_line(&mut topic)?;

            println!("Enter the message to publish:");
            let mut message = String::new();
            std::io::stdin().read_line(&mut message)?;

            let mut levels = vec![];

            for level in topic.split(' ') {
                if let Ok(TopicLevel::Literal(literal)) =
                    TopicLevel::from_bytes(level.as_bytes().to_vec())
                {
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

            // Send Publish packet
            println!("Packet Publish: {:?}", publish_packet);

            let _ = to_server_stream.write(publish_packet.to_bytes().as_slice());
            println!("Sent Publish packet");
        }
        if command == "unsubscribe" {
            println!("Enter the topic to unsubscribe to (sepataded by spaces):");
            let mut topic = String::new();
            std::io::stdin().read_line(&mut topic)?;

            let mut levels = vec![];
            for level in topic.split(' ') {
                if let Ok(topic_level) = TopicLevel::from_bytes(level.as_bytes().to_vec()) {
                    levels.push(topic_level);
                }
            }

            let topic_filter = TopicFilter::new(levels, false);
            let packet_id = 1;

            let topics_filters = vec![(topic_filter)];

            let unsubscribe_packet = Unsubscribe::new(packet_id, topics_filters);

            // Send Subscribe packet
            println!("Packet ID: {:?}", unsubscribe_packet.packet_identifier);
            println!("Topics: {:?}", unsubscribe_packet.topics);
            let _ = to_server_stream.write(unsubscribe_packet.to_bytes().as_slice());
            println!("Sent UnSubscribe packet");
        }
        if command == "disconnect" {
            let disconnect_packet = Disconnect::new();
            println!("Attempting disconnection!");
            let _ = to_server_stream.write(disconnect_packet.to_bytes().as_slice());
        }
        if command == "ping" {
            let pingreq_packet = Pingreq::new();
            println!("Sending ping!");
            println!("Packet: {:?}", pingreq_packet);

            let _ = to_server_stream.write(pingreq_packet.to_bytes().as_slice());
        }
    }
    Ok(())
}
