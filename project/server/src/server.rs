use std::{
    collections::HashMap, io::Write, net::{TcpListener, TcpStream}, sync::{mpsc, RwLock}, thread
};

pub use sauron::model::{
    packet::Packet,
    packets::{
        connect::Connect, disconnect::Disconnect, pingresp::Pingresp, puback::Puback,
        publish::Publish, subscribe::Subscribe, unsubscribe::Unsubscribe,
    },
};

use super::topic_handler::TopicHandlerTask;
use super::config::Config;
use super::topic_handler::TopicHandler;

pub struct Server {
    config: Option<Config>,
    // Channel for client actions
    client_actions_sender: mpsc::Sender<TopicHandlerTask>,
    client_actions_receiver: mpsc::Receiver<TopicHandlerTask>,
    // Map to store client senders for communication
    client_senders: RwLock<HashMap<Vec<u8>, mpsc::Sender<Publish>>>,
}

impl Server {
    pub fn new() -> Self {
        let (client_actions_sender, client_actions_receiver) = mpsc::channel();
        Server {
            config: Config::new("/."),
            client_actions_sender,
            client_actions_receiver,
            client_senders: RwLock::new(HashMap::new()),
        }
    }

    pub fn server_run(&self, address: &str) -> std::io::Result<()> {
        let server = Server::new();
        let listener = TcpListener::bind(address)?; 
        Server::initialize_topic_handler_thread(server.client_actions_receiver);

        for stream_result in listener.incoming() {
            match stream_result {
                Ok(stream) => {
                    self.handle_new_connection(stream)?;
                }
                Err(err) => {
                    println!("Error accepting connection: {:?}", err);
                }
            }
        }

        Ok(())
    }

    pub fn handle_new_connection(&self, mut stream: TcpStream) -> std::io::Result<()> {
        
        let packet = match Packet::from_bytes(&mut stream) {
            Ok(packet) => packet,
            Err(err) => {
                println!("Error reading packet: {:?}", err);
                return Ok(());
            }
        };
        Ok(())
    }

    pub fn initialize_topic_handler_thread(client_actions_receiver: mpsc::Receiver<TopicHandlerTask>) {
        thread::spawn(move || {
            let topic_handler = TopicHandler::new(client_actions_receiver);
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
        let (client_sender, client_receiver) = mpsc::channel(); // Create a channel for this client

        let mut client_senders = self.client_senders.write().unwrap();
        client_senders.insert(client_id.clone(), client_sender);

        //let new_client = Client::new(client_id.clone(), "PASSWORD".to_string(), stream, true, 0);
    
        // self.client_actions_sender
        //     .send(TopicHandlerTask::ClientConnected(new_client))
        //     .unwrap();

        println!("New client connected: {:?}", client_id);
        self.create_new_client_thread(
            self.client_actions_sender.clone(),
            stream,
            client_id,
            client_receiver, 
        );
    }

    pub fn create_new_client_thread(
        &self,
        sender_to_topics_channel: std::sync::mpsc::Sender<TopicHandlerTask>,
        mut stream: TcpStream,
        client_id: Vec<u8>,
        client_receiver: mpsc::Receiver<Publish>, // Receive Publish packets
    ) {
        thread::spawn(move || {
            let address = stream.peer_addr().unwrap().to_string();
            let mut maintain_thread = true;

            while maintain_thread {
                
                //let packet = Packet::from_bytes(&mut stream);

                let _ = match Packet::from_bytes(&mut stream) {
                    Ok(packet) => {
                        maintain_thread = handle_packet(
                            packet,
                            client_id.clone(),
                            stream.try_clone().unwrap(), // Clone the stream for sending
                            sender_to_topics_channel.clone(), // Clone the sender channel
                        );
                    },
                    Err(err) => {
                        println!("Error reading packet: {:?}", err);
                        break;
                    }
                };

                match client_receiver.try_recv() {
                    Ok(publish_packet) => {
                        println!("Received publish packet: {:?}", publish_packet);
                        // Send the Publish packet back to the client
                        stream.write_all(&publish_packet.to_bytes()).unwrap();
                    }
                    Err(_) => {} // Do nothing if there is no message to receive
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
    let maintain_thread = match packet {
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
    maintain_thread
}

pub fn handle_publish(
    publish_packet: Publish,
    sender_to_topics_channel: std::sync::mpsc::Sender<TopicHandlerTask>,
    client_id: Vec<u8>,
) -> bool {
    sender_to_topics_channel
        .send(TopicHandlerTask::Publish(publish_packet, client_id))
        .unwrap();
    true
}

pub fn handle_puback(
    puback_packet: Puback,
    sender_to_topics_channel: std::sync::mpsc::Sender<TopicHandlerTask>,
    client_id: Vec<u8>,
) -> bool {
    // sender_to_topics_channel
    //     .send(TopicHandlerTask::RegisterPubAck(puback_packet))
    //     .unwrap();

    true
}

pub fn handle_subscribe(
    subscribe_packet: Subscribe,
    sender_to_topics_channel: std::sync::mpsc::Sender<TopicHandlerTask>,
    client_id: Vec<u8>,
) -> bool {
    sender_to_topics_channel
        .send(TopicHandlerTask::SubscribeClient(
            subscribe_packet,
            client_id,
        ))
        .unwrap();

    true
}

pub fn handle_unsubscribe(
    unsubscribe_packet: Unsubscribe,
    sender_to_topics_channel: std::sync::mpsc::Sender<TopicHandlerTask>,
    client_id: Vec<u8>,
) -> bool {
    //if !validate_client_id(unsubscribe_packet, client_id) {return false};
    sender_to_topics_channel
        .send(TopicHandlerTask::UnsubscribeClient(
            unsubscribe_packet,
            client_id,
        ))
        .unwrap();

    true
}

pub fn handle_disconnect(
    packet: Disconnect,
    sender_to_topics_channel: std::sync::mpsc::Sender<TopicHandlerTask>,
    client_id: Vec<u8>,
) -> bool {
    sender_to_topics_channel
        .send(TopicHandlerTask::ClientDisconnected(client_id))
        .unwrap();

    false
}

pub fn handle_pingreq(stream: TcpStream) -> bool {
    let pingresp_packet = Pingresp::new();
    let pingresp_bytes = pingresp_packet.to_bytes();
    //stream.write_all(&pingresp_bytes).unwrap();

    true
}
