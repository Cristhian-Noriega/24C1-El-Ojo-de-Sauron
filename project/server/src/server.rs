#![allow(unused_variables)]
#![allow(dead_code)]

use std::{
    collections::HashMap,
    io::Write,
    net::{TcpListener, TcpStream},
    sync::{mpsc, RwLock},
    thread,
};

pub use sauron::model::{
    packet::Packet,
    packets::{
        connack::Connack, connect::Connect, disconnect::Disconnect, pingresp::Pingresp,
        puback::Puback, publish::Publish, subscribe::Subscribe, unsubscribe::Unsubscribe,
    },
    return_codes::connect_return_code::ConnectReturnCode,
};

use crate::client::Client;

use super::config::Config;
use super::task_handler::Task;
use super::task_handler::TaskHandler;

pub struct Server {
    config: Config,
    // Channel for client actions
    client_actions_sender: mpsc::Sender<Task>,
    // Map to store client senders for communication
    client_senders: RwLock<HashMap<Vec<u8>, mpsc::Sender<Publish>>>,
    // Task handler to handle client actions
    // task_handler: TaskHandler,
}

impl Server {
    pub fn new(config: Config) -> Self {
        let (client_actions_sender, client_actions_receiver) = mpsc::channel();
        let task_handler = TaskHandler::new(client_actions_receiver);
        task_handler.initialize_task_handler_thread();
        Server {
            config,
            client_actions_sender,
            client_senders: RwLock::new(HashMap::new()),
            
        }
    }

    pub fn server_run(&self) -> std::io::Result<()> {
        let address = format!("{}:{}", self.config.get_address(), self.config.get_port());
        println!("Server running on address: {}", address);
        let server = Server::new(self.config.clone());
        let listener = TcpListener::bind(&address)?;
        //Server::initialize_task_handler_thread(self.task_handler);

        for stream_result in listener.incoming() {
            match stream_result {
                Ok(stream) => {
                    println!("New connection");
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
        match Packet::from_bytes(&mut stream) {
            Ok(packet) => self.handle_incoming_packet(packet, stream),
            Err(err) => {
                println!("Error parsing packet: {:?}", err);
                return Ok(());
            }
        }
        Ok(())
    }

    pub fn initialize_task_handler_thread(task_handler: TaskHandler) {
        println!("Starting task handler thread\n");
        std::thread::spawn(move || {
            task_handler.run();
        });
    }

    pub fn handle_incoming_packet(&self, packet: Packet, stream: TcpStream) {
        match packet {
            Packet::Connect(connect_packet) => self.connect_new_client(connect_packet, stream),
            //Packet::Subscribe(subscribe_packet) => handle_packet(subscribe_packet, stream),
            _ => println!("Unsupported packet type"),
        }
    }

    pub fn connect_new_client(&self, connect_packet: Connect, mut stream: TcpStream) {
        println!("Received Connect Package");
        let client_id = connect_packet.client_id().content();
        let (client_sender, client_receiver) = mpsc::channel(); // Create a channel for this client

        let mut client_senders = self.client_senders.write().unwrap();
        client_senders.insert(client_id.clone(), client_sender);

        //let new_client = Client::new(client_id.clone(), "PASSWORD".to_string(), stream, true, 0);

        // self.client_actions_sender
        //     .send(TopicHandlerTask::ClientConnected(new_client))
        //     .unwrap();

        println!(
            "New client connected: {:?}",
            String::from_utf8_lossy(client_id)
        );

        let connack_packet = Connack::new(false, ConnectReturnCode::ConnectionAccepted);
        let _ = stream.write(connack_packet.to_bytes().as_slice());

        self.create_new_client_thread(
            self.client_actions_sender.clone(),
            stream,
            client_id.clone(),
            client_receiver,
        );
    }

    pub fn create_new_client_thread(
        &self,
        sender_to_task_channel: std::sync::mpsc::Sender<Task>,
        mut stream: TcpStream,
        client_id: Vec<u8>,
        client_receiver: mpsc::Receiver<Publish>,
    ) {
        thread::spawn(move || {
            println!("Welcome to the newly connected client thread\n");

            loop {
                let packet = Packet::from_bytes(&mut stream);
                match packet {
                    Ok(packet) => {
                        println!("Received packet: {:?}", packet);

                        let handling_result = handle_packet(
                            packet,
                            client_id.clone(),
                            &mut stream,
                            sender_to_task_channel.clone(),
                        );

                        if !handling_result {
                            println!("Connection closed");
                            break;
                        };
                    }
                    Err(err) => {
                        println!("Error parsing packet: {:?}", err);
                        println!("Connection closed");
                        break;
                    }
                }
            }
        });
    }
}

pub fn handle_packet(
    packet: Packet,
    client_id: Vec<u8>,
    stream: &mut TcpStream,
    sender_to_topics_channel: std::sync::mpsc::Sender<Task>,
) -> bool {
    //println!("packet {:?}", packet);
    match packet {
        Packet::Publish(publish_packet) => {
            println!("Received Publish packet");
            handle_publish(publish_packet, sender_to_topics_channel, client_id)
        }
        Packet::Puback(puback_packet) => {
            handle_puback(puback_packet, sender_to_topics_channel, client_id)
        }
        Packet::Subscribe(subscribe_packet) => {
            println!("Received Subscribe packet");
            handle_subscribe(subscribe_packet, sender_to_topics_channel, client_id)
        }
        Packet::Unsubscribe(unsubscribe_packet) => {
            println!("Received Unsubscribe packet");
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
    }
}

pub fn handle_connect(sender_to_topics_channel: std::sync::mpsc::Sender<Task>, client: Client) -> bool {
    sender_to_topics_channel
        .send(Task::ClientConnected(client))
        .unwrap();
    true
}

pub fn handle_publish(
    publish_packet: Publish,
    sender_to_topics_channel: std::sync::mpsc::Sender<Task>,
    client_id: Vec<u8>,
) -> bool {
    sender_to_topics_channel
        .send(Task::Publish(publish_packet, client_id))
        .unwrap();
    true
}

pub fn handle_puback(
    puback_packet: Puback,
    sender_to_topics_channel: std::sync::mpsc::Sender<Task>,
    client_id: Vec<u8>,
) -> bool {
    // sender_to_topics_channel
    //     .send(TopicHandlerTask::RegisterPubAck(puback_packet))
    //     .unwrap();

    true
}

pub fn handle_subscribe(
    subscribe_packet: Subscribe,
    sender_to_task_channel: std::sync::mpsc::Sender<Task>,
    client_id: Vec<u8>,
) -> bool {
    sender_to_task_channel
        .send(Task::SubscribeClient(subscribe_packet, client_id))
        .unwrap();

    true
}

pub fn handle_unsubscribe(
    unsubscribe_packet: Unsubscribe,
    sender_to_task_channel: std::sync::mpsc::Sender<Task>,
    client_id: Vec<u8>,
) -> bool {
    sender_to_task_channel
        .send(Task::UnsubscribeClient(unsubscribe_packet, client_id))
        .unwrap();

    true
}

pub fn handle_disconnect(
    packet: Disconnect,
    sender_to_task_channel: std::sync::mpsc::Sender<Task>,
    client_id: Vec<u8>,
) -> bool {
    sender_to_task_channel
        .send(Task::ClientDisconnected(client_id))
        .unwrap();

    false
}

pub fn handle_pingreq(stream: &mut TcpStream) -> bool {
    println!("Received Pingreq packet");
    let pingresp_packet = Pingresp::new();
    let pingresp_bytes = pingresp_packet.to_bytes();
    stream.write_all(&pingresp_bytes).unwrap();
    true
}
