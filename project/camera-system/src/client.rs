use std::io::ErrorKind;
use std::io::Write;
use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;
use std::thread;

pub use mqtt::model::{
    components::{qos::QoS, topic_filter::TopicFilter, topic_level::TopicLevel},
    packet::Packet,
    packets::{
        connect::Connect, disconnect::Disconnect, pingreq::Pingreq, publish::Publish,
        subscribe::Subscribe, unsubscribe::Unsubscribe,
    },
};

use mqtt::model::components::encoded_string::EncodedString;
use mqtt::model::components::topic_name::TopicName;

pub fn client_run(address: &str, actions_input: &mut dyn Read) -> std::io::Result<()> {
    let reader = BufReader::new(actions_input);

    let mut to_server_stream = connect_to_server(address)?;

    let mut read_incoming_packages = to_server_stream.try_clone()?;
    thread::spawn(move || loop {
        let packet = match Packet::from_bytes(&mut read_incoming_packages) {
            Ok(packet) => packet,
            Err(_) => return,
        };

        match packet {
            Packet::Publish(publish) => {
                let message = publish.message();
                let message_str = String::from_utf8_lossy(message).to_string();

                println!("Received Publish packet!");
                println!("Message: {:?}", message_str);
            }
            Packet::Puback(_puback) => {
                println!("Received Puback packet\n");
            }
            Packet::Pingresp(_pingresp) => {
                println!("Received ping Response!\n");
            }
            Packet::Suback(_suback) => {
                println!("Received Suback packet\n");
            }
            Packet::Unsuback(_unsuback) => {
                println!("Received Unsuback packet\n");
            }
            _ => println!("Received unsupported packet type\n"),
        }
    });

    for line in reader.lines().map_while(Result::ok) {
        let command = line.trim();
        if command == "subscribe" {
            println!("Enter the topic to subscribe to:");
            let mut topic = String::new();
            std::io::stdin().read_line(&mut topic)?;

            topic = topic.trim_end_matches('\n').to_string();
            topic = topic.trim_end_matches('\r').to_string();

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
            println!("Packet ID: {:?}", subscribe_packet.packet_identifier());
            //println!("Topics: {:?}", subscribe_packet.topics());
            let _ = to_server_stream.write(subscribe_packet.to_bytes().as_slice());
            println!("Sent Subscribe packet");
        }
        if command == "publish" {
            println!("Enter the topic to publish to:");
            let mut topic = String::new();
            std::io::stdin().read_line(&mut topic)?;

            topic = topic.trim_end_matches('\n').to_string();
            topic = topic.trim_end_matches('\r').to_string();

            println!("Enter the message to publish:");
            let mut message = String::new();
            std::io::stdin().read_line(&mut message)?;
            message = message.trim().to_string();

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
            //println!("Packet Publish: {:?}", publish_packet);

            let _ = to_server_stream.write(publish_packet.to_bytes().as_slice());
            println!(
                "Sent Publish packet to topic: {:?} with message: {:?}",
                topic, message
            );
        }
        if command == "unsubscribe" {
            println!("Enter the topic to unsubscribe to:");
            let mut topic = String::new();
            std::io::stdin().read_line(&mut topic)?;

            topic = topic.trim_end_matches('\n').to_string();
            topic = topic.trim_end_matches('\r').to_string();

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
            println!("Packet ID: {:?}", unsubscribe_packet.packet_identifier());
            //println!("Topics: {:?}", unsubscribe_packet.topics);
            let _ = to_server_stream.write(unsubscribe_packet.to_bytes().as_slice());
            println!("Sent UnSubscribe packet");
        }
        if command == "disconnect" {
            let disconnect_packet = Disconnect::new();
            println!("Attempting disconnection!");
            let _ = to_server_stream.write(disconnect_packet.to_bytes().as_slice());
            println!("Disconnected from server!");
        }
        if command == "ping" {
            let pingreq_packet = Pingreq::new();
            println!("Sending ping!");

            let _ = to_server_stream.write(pingreq_packet.to_bytes().as_slice());
        }
        if command == "connect" {
            to_server_stream = connect_to_server(address)?;
        }
    }
    Ok(())
}

pub fn connect_to_server(address: &str) -> std::io::Result<TcpStream> {
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
