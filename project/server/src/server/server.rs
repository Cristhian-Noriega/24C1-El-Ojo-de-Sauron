#![allow(dead_code)]
#![allow(unused_variables)]
use crate::{
    client::Client,
    config::Config,
    topic_handler::{TopicHandler, TopicHandlerTask},
};
use std::{
    io::{BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    sync::mpsc,
    thread,
};

pub use sauron::model::{
    packet::Packet,
    packets::{
        connect::Connect, disconnect::Disconnect, pingresp::Pingresp, puback::Puback,
        publish::Publish, subscribe::Subscribe, unsubscribe::Unsubscribe,
    },
};

pub struct Server {
    config: Option<Config>,
    client_actions_sender_channel: mpsc::Sender<TopicHandlerTask>,
    client_actions_receiver_channel: mpsc::Receiver<TopicHandlerTask>,
}

impl Server {
    pub fn new() -> Self {
        let (client_actions_sender_channel, client_actions_receiver_channel) = mpsc::channel();
        Server {
            config: Config::new("/."),
            client_actions_sender_channel,
            client_actions_receiver_channel,
        }
    }

    pub fn server_run(self, address: &str) -> std::io::Result<()> {
        let server = Server::new();
        let listener = TcpListener::bind(address)?;

        Server::intialize_topic_handler_thread(server.client_actions_receiver_channel);

        for stream_result in listener.incoming() {
            match stream_result {
                Ok(stream) => {
                    let address = stream.peer_addr()?.to_string();
                    println!("Nuevo paquete de la dirección: {:?}", address);
                    let mut reader = BufReader::new(stream);
                    let mut buffer = Vec::new();



                    let mut cursor = std::io::Cursor::new(buffer);

                    let _ = match Packet::from_bytes(&mut cursor) {
                        Ok(packet) => {
                            println!("Packet recibido desde la dirección: {:?}", address);
                            server.handle_incoming_packet(packet, stream);
                        }
                        Err(e) => {
                            println!("Error al deserializar el paquete: {:?}", e);
                        }
                    };
                    // println!("Packet recibidio desde la dirección: {:?}", address);
                    // server.handle_incoming_packet(packet, stream);
                }
                Err(err) => {
                    println!("Error al recibir paquete: {:?}", err);
                }
            }
        }

        Ok(())
    }
    pub fn intialize_topic_handler_thread(
        client_actions_receiver_channel: std::sync::mpsc::Receiver<TopicHandlerTask>,
    ) {
        thread::spawn(move || {
            let topic_handler = TopicHandler::new(client_actions_receiver_channel);
            topic_handler.run();
        });
    }

    pub fn handle_incoming_packet(&self, packet: Packet, stream: TcpStream) {
        match packet {
            Packet::Connect(connect_packet) => self.connect_new_client(connect_packet, stream),
            _ => println!("Unsupported packet type"),
        }
    }

    pub fn connect_new_client(&self, connect_packet: Connect, stream: TcpStream) {
        let client_id = connect_packet.client_id.content;
        let new_client = Client::new(client_id.clone(), "PASSWORD".to_string(), stream, true, 0);
        self.client_actions_sender_channel
            .send(TopicHandlerTask::ClientConnected(new_client));

        println!("New client connected: {:?}", client_id);
        self.create_new_client_thread(self.client_actions_sender_channel, stream, client_id); 
    }

    pub fn create_new_client_thread(
        &self,
        sender_to_topics_channel: std::sync::mpsc::Sender<TopicHandlerTask>,
        mut stream: TcpStream,
        client_id: Vec<u8>,
    ) {
        thread::spawn(move || {
            let address = stream.peer_addr().unwrap().to_string();
            let mut mantain_thread = true;

            while mantain_thread {
                let mut reader = BufReader::new(&mut stream);
                let mut buffer = Vec::new();

                match reader.read_to_end(&mut buffer) {
                    Ok(_) => {
                        let mut cursor = std::io::Cursor::new(buffer);
                        //let packet: Packet = Packet::from_bytes(&mut cursor)?;
                        let _ = match Packet::from_bytes(&mut cursor) {
                            Ok(packet) => {
                                println!("Packet received from address: {:?}", address);
                                mantain_thread = handle_packet(
                                    packet,
                                    client_id,
                                    stream,
                                    sender_to_topics_channel,
                                );
                                buffer.clear();
                            }
                            Err(err) => {
                                println!("Error reading from stream: {:?}", err);
                                mantain_thread = false;
                            }
                        };
                    }
                    Err(err) => {
                        println!("Error reading from stream: {:?}", err);
                        mantain_thread = false;
                    }
                }
            }
        });
    }
}

pub fn handle_packet(
    packet: Packet,
    client_id: Vec<u8>,
    stream: TcpStream,
    sender_to_topics_channel: std::sync::mpsc::Sender<TopicHandlerTask>,
) -> bool {
    let mantain_thread = match packet {
        Packet::Publish(publish_packet) => {
            handle_publish(publish_packet, sender_to_topics_channel, client_id)
        }
        Packet::Puback(puback_packet) => {
            handle_puback(puback_packet, sender_to_topics_channel, client_id)
        }
        Packet::Subscribe(subscribe_packet) => {
            handle_subscribe(subscribe_packet, sender_to_topics_channel, client_id)
        }
        Packet::Unsubscribe(unsubscribe_packet) => {
            handle_unsubscribe(unsubscribe_packet, sender_to_topics_channel, client_id)
        }
        Packet::Pingreq(pingreq_packet) => handle_pingreq(stream),
        Packet::Disconnect(disconnect_packet) => {
            handle_disconnect(disconnect_packet, sender_to_topics_channel, client_id)
        }
        _ => {
            println!("Unsupported packet type");
            false
        }
    };
    mantain_thread
}
// Validates that the client_id in the packet is the same as the client_id of the current thread. If it isn't the same, the thread should be killed.
// solo el paquete connect tiene el client id
// pub fn validate_client_id(packet: Packet, client_id: Vec<u8>) -> bool {
//     if packet.client_id != client_id{
//         println!("Client ID doesn't match.");
//         return false;
//     }
//     true
// }

pub fn handle_publish(
    publish_packet: Publish,
    sender_to_topics_channel: std::sync::mpsc::Sender<TopicHandlerTask>,
    client_id: Vec<u8>,
) -> bool {
    //validate_client_id(publish_packet, client_id);
    sender_to_topics_channel.send(TopicHandlerTask::Publish(publish_packet, client_id));

    true
}

pub fn handle_puback(
    puback_packet: Puback,
    sender_to_topics_channel: std::sync::mpsc::Sender<TopicHandlerTask>,
    client_id: Vec<u8>,
) -> bool {
    //if !validate_client_id(puback_packet, client_id) {return false};
    sender_to_topics_channel.send(TopicHandlerTask::RegisterPubAck(puback_packet));

    true
}

pub fn handle_subscribe(
    subscribe_packet: Subscribe,
    sender_to_topics_channel: std::sync::mpsc::Sender<TopicHandlerTask>,
    client_id: Vec<u8>,
) -> bool {
    //if !validate_client_id(subscribe_packet, client_id) {return false};
    sender_to_topics_channel.send(TopicHandlerTask::SubscribeClient(
        subscribe_packet,
        client_id,
    ));

    true
}

pub fn handle_unsubscribe(
    unsubscribe_packet: Unsubscribe,
    sender_to_topics_channel: std::sync::mpsc::Sender<TopicHandlerTask>,
    client_id: Vec<u8>,
) -> bool {
    //if !validate_client_id(unsubscribe_packet, client_id) {return false};
    sender_to_topics_channel.send(TopicHandlerTask::UnsubscribeClient(
        unsubscribe_packet,
        client_id,
    ));

    true
}

pub fn handle_disconnect(
    packet: Disconnect,
    sender_to_topics_channel: std::sync::mpsc::Sender<TopicHandlerTask>,
    client_id: Vec<u8>,
) -> bool {
    //sender_to_topics_channel.send(TopicHandlerTask::DisconnectClient(client_id));

    false
}

pub fn handle_pingreq(stream: TcpStream) -> bool {
    let pingresp_packet = Pingresp::new();
    let pingresp_bytes = pingresp_packet.to_bytes();
    stream.write_all(&pingresp_bytes);

    true
}
