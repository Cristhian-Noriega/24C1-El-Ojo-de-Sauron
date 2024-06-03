use std::env::args;
use std::io::stdin;

pub use mqtt::model::{
    components::{qos::QoS, topic_filter::TopicFilter, topic_level::TopicLevel},
    packet::Packet,
    packets::{
        connect::Connect, disconnect::Disconnect, pingreq::Pingreq, puback::Puback,
        publish::Publish, subscribe::Subscribe, unsubscribe::Unsubscribe,
    },
};

mod camera;
mod camera_status;
mod camera_system;
mod client;
mod incident;

const CLIENT_ARGS: usize = 3;

fn main() {
    let argv = args().collect::<Vec<String>>();
    if argv.len() != CLIENT_ARGS {
        println!("Cantidad de argumentos inválidos");
        let app_name = &argv[0];
        println!("{:?} <host> <puerto>", app_name);

        return;
    }

    let address = argv[1].clone() + ":" + &argv[2];

    if let Err(e) = client::client_run(&address, &mut stdin()) {
        println!("Error: {:?}", e);
    }
}

//the client receives a connack packet from the server
