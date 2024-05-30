#![allow(unused_variables)]
#![allow(dead_code)]

use std::{
    collections::HashMap,
    net::{TcpListener, TcpStream},
    sync::{mpsc, Arc, RwLock},
    thread,
};

pub use mqtt::model::{
    packet::Packet,
    packets::{
        connect::Connect, puback::Puback, publish::Publish, subscribe::Subscribe,
        unsubscribe::Unsubscribe,
    },
};

use crate::client::Client;

use super::config::Config;
use super::task_handler::Task;
use super::task_handler::TaskHandler;
use super::logfile::Logger;

pub struct Server {
    config: Config,
    // Channel for client actions
    client_actions_sender: mpsc::Sender<Task>,
    // Map to store client senders for communication
    client_senders: RwLock<HashMap<Vec<u8>, mpsc::Sender<Publish>>>,
    // Task handler to handle client actions
    // task_handler: TaskHandler,
    log_file: Arc<Logger>
}

impl Server {
    pub fn new(config: Config) -> Self {
        let (client_actions_sender, client_actions_receiver) = mpsc::channel();
        
        let log_file = Arc::new(Logger::new(&config.get_log_file()));
        let task_handler = TaskHandler::new(client_actions_receiver, log_file.clone());
        task_handler.initialize_task_handler_thread();
        Server {
            config,
            client_actions_sender,
            client_senders: RwLock::new(HashMap::new()),
            log_file: log_file,
            
        }
    }

    pub fn server_run(&self) -> std::io::Result<()> {
        let address = format!("{}:{}", self.config.get_address(), self.config.get_port());
        //println!("Server running on address: {}\n", address);
        //self.log_file.log(&format!("Server running on address: {}\n", address));
        self.log_file.log("INFO", &format!("Server running on address: {}", address));
        let listener = TcpListener::bind(&address)?;

        for stream_result in listener.incoming() {
            match stream_result {
                Ok(stream) => {
                    //println!("New connection received");
                    self.log_file.log("INFO", "New connection received");
                    self.handle_new_connection(stream)?;
                }
                Err(err) => {
                    self.log_file.log("ERROR", &format!("Error accepting connection: {:?}", err))
                    //println!("Error accepting connection: {:?}", err);
                    //self.log_file.lock().unwrap().log(&format!("Error accepting connection: {:?}", err));
                }
            }
        }

        Ok(())
    }

    pub fn handle_new_connection(&self, mut stream: TcpStream) -> std::io::Result<()> {
        match Packet::from_bytes(&mut stream) {
            Ok(packet) => self.handle_incoming_packet(packet, stream),
            Err(err) => {
                self.log_file.log("ERROR", &format!("Error reading packet: {:?}", err));
                //println!("Error reading packet: {:?}", err);
                //self.log_file.lock().unwrap().log(&format!("Error reading packet: {:?}", err));
                return Ok(());
            }
        }
        Ok(())
    }

    pub fn initialize_task_handler_thread(task_handler: TaskHandler) {
        //println!("Starting task handler thread");
        std::thread::spawn(move || {
            task_handler.run();
        });
    }

    pub fn handle_incoming_packet(&self, packet: Packet, stream: TcpStream) {
        match packet {
            Packet::Connect(connect_packet) => self.connect_new_client(connect_packet, stream),
            _ => println!("Unsupported packet type"),
        }
    }

    pub fn connect_new_client(&self, connect_packet: Connect, stream: TcpStream) {
        //add more info to the connect message
        let message = format!("Received Connect Packet from client with ID: {}", connect_packet.client_id());
        self.log_file.log("INFO", message.as_str());
        //println!("Received Connect Package");
        //self.log_file.lock().unwrap().log("Received Connect Package");
        let client_id = connect_packet.client_id().content();

        let new_client = Client::new(
            client_id.clone(),
            "PASSWORD".to_string(),
            stream.try_clone().unwrap(),
            true,
            0,
        );
        handle_connect(self.client_actions_sender.clone(), new_client);

        self.create_new_client_thread(
            self.client_actions_sender.clone(),
            stream,
            client_id.clone(),
            self.log_file.clone(),
        );
    }

    pub fn create_new_client_thread(
        &self,
        sender_to_task_channel: std::sync::mpsc::Sender<Task>,
        mut stream: TcpStream,
        client_id: Vec<u8>,
        log_file: Arc<Logger>,
    ) {
        thread::spawn(move || {
            //println!("New client thread initiated\n");
            log_file.log("INFO", "New client thread initiated");
            loop {
                let packet = Packet::from_bytes(&mut stream);
                match packet {
                    Ok(packet) => {
                        //println!("Received new packet");
                        log_file.log("INFO", "Received new packet");

                        let handling_result = handle_packet(
                            packet,
                            client_id.clone(),
                            &mut stream,
                            sender_to_task_channel.clone(),
                            log_file.clone(),
                        );

                        if !handling_result {
                            break;
                        };
                    }
                    Err(err) => {
                        //println!("Connection Error: {:?}", err);
                        log_file.log("ERROR", &format!("Connection Error: {:?}", err));
                        break;
                    }
                }
            }
            //disconnect_client(sender_to_task_channel, client_id);
            println!("Connection closed");
            log_file.log("INFO", "Connection closed");
        });
    }
}

pub fn handle_packet(
    packet: Packet,
    client_id: Vec<u8>,
    stream: &mut TcpStream,
    sender_to_task_channel: std::sync::mpsc::Sender<Task>,
    log_file: Arc<Logger>,
) -> bool {
    match packet {
        Packet::Publish(publish_packet) => {
            // println!(
            //     "Received Publish packet from client: {:?}",
            //     std::str::from_utf8(&client_id).unwrap()
            // );
            log_file.log("INFO", &format!(
                "Received Publish packet from client: {}",
                String::from_utf8_lossy(&client_id)
            ));
            handle_publish(publish_packet, sender_to_task_channel, client_id)
        }
        Packet::Puback(puback_packet) => {
            // println!(
            //     "Received Puback packet from client: {:?}",
            //     std::str::from_utf8(&client_id).unwrap()
            // );
            log_file.log("INFO", &format!(
                "Received Puback packet from client: {}",
                String::from_utf8_lossy(&client_id)
            ));
            handle_puback(puback_packet, sender_to_task_channel, client_id)
        }
        Packet::Subscribe(subscribe_packet) => {
            // println!(
            //     "Received Subscribe packet from client: {:?}",
            //     std::str::from_utf8(&client_id).unwrap()
            // );
            log_file.log("INFO", &format!(
                "Received Subscribe packet from client: {}",
                String::from_utf8_lossy(&client_id)
            ));
            handle_subscribe(subscribe_packet, sender_to_task_channel, client_id)
        }
        Packet::Unsubscribe(unsubscribe_packet) => {
            // println!(
            //     "Received Unsubscribe packet from client: {:?}",
            //     std::str::from_utf8(&client_id).unwrap()
            // );
            log_file.log("INFO", &format!(
                "Received Unsubscribe packet from client: {}",
                String::from_utf8_lossy(&client_id)
            ));
            handle_unsubscribe(unsubscribe_packet, sender_to_task_channel, client_id)
        }
        Packet::Pingreq(pingreq_packet) => {
            // println!(
            //     "Received Pingreq packet from client: {:?}",
            //     std::str::from_utf8(&client_id).unwrap()
            // );
            log_file.log("INFO", &format!(
                "Received Pingreq packet from client: {}",
                String::from_utf8_lossy(&client_id)
            ));
            handle_pingreq(sender_to_task_channel, client_id)
        }
        Packet::Disconnect(disconnect_packet) => {
            // println!(
            //     "Received Disconnect packet from client: {:?}",
            //     std::str::from_utf8(&client_id).unwrap()
            // );
            log_file.log("INFO", &format!(
                "Received Disconnect packet from client: {}",
                String::from_utf8_lossy(&client_id)
            ));
            disconnect_client(sender_to_task_channel, client_id)
        }
        _ => {
            // println!("Received an unsupported packet type");
            // println!("Closing connection");
            log_file.log("INFO", "Received an unsupported packet type");
            log_file.log("INFO", "Closing connection");
            disconnect_client(sender_to_task_channel, client_id);
            false
        }
    }
}

pub fn handle_connect(
    sender_to_topics_channel: std::sync::mpsc::Sender<Task>,
    client: Client,
) -> bool {
    sender_to_topics_channel
        .send(Task::ConnectClient(client))
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

pub fn handle_pingreq(
    sender_to_task_channel: std::sync::mpsc::Sender<Task>,
    client_id: Vec<u8>,
) -> bool {
    sender_to_task_channel
        .send(Task::RespondPing(client_id))
        .unwrap();
    true
}

pub fn disconnect_client(
    sender_to_task_channel: std::sync::mpsc::Sender<Task>,
    client_id: Vec<u8>,
) -> bool {
    sender_to_task_channel
        .send(Task::DisconnectClient(client_id))
        .unwrap();
    false
}
