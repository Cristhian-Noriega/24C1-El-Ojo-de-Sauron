#![allow(unused_variables)]
#![allow(dead_code)]

use std::{
    collections::HashMap,
    net::{TcpListener, TcpStream},
    sync::{
        mpsc::{self, Sender},
        Arc, RwLock,
    },
    thread,
};

pub use mqtt::model::{
    packet::Packet,
    packets::{
        connect::Connect, puback::Puback, publish::Publish, subscribe::Subscribe,
        unsubscribe::Unsubscribe,
    },
};

use crate::{client::Client, client_manager::ClientManager};

use super::{
    config::Config,
    logfile::Logger,
    task_handler::{Task, TaskHandler},
};

/// Represents the MQTT server that will be handling all messages
pub struct Server {
    config: Config,
    // Channel for client actions
    client_actions_sender: Sender<Task>,
    // Map to store client senders for communication
    client_senders: RwLock<HashMap<Vec<u8>, Sender<Publish>>>,
    log_file: Arc<Logger>,
    //registered_clients: Arc<Mutex<HashMap<(Vec<u8>, Vec<u8>), bool>>>,
    client_manager: Arc<RwLock<ClientManager>>,
}

impl Server {
    /// Creates a new server with the specified configuration
    pub fn new(config: Config) -> Self {
        let (client_actions_sender, client_actions_receiver) = mpsc::channel();

        let log_file = Arc::new(Logger::new(config.get_log_file()));
        let client_manager = ClientManager::new();
        client_manager.make_initial_registrations(config.clone());
        let client_manager = Arc::new(RwLock::new(client_manager));

        let task_handler = TaskHandler::new(
            client_actions_receiver,
            log_file.clone(),
            client_manager.clone(),
        );

        task_handler.initialize_task_handler_thread();

        Server {
            config,
            client_actions_sender,
            client_senders: RwLock::new(HashMap::new()),
            log_file,
            client_manager,
        }
    }

    /// Starts the server
    pub fn server_run(&self) -> std::io::Result<()> {
        let address = self.config.get_address();

        self.log_file
            .info(&format!("Server running on address: {}\n", address));
        let listener = TcpListener::bind(address)?;

        for stream_result in listener.incoming() {
            match stream_result {
                Ok(stream) => {
                    self.log_file.info("New connection received");
                    self.handle_new_connection(stream)?;
                }
                Err(err) => {
                    self.log_file
                        .error(&format!("Error accepting connection: {:?}", err));
                }
            }
        }

        Ok(())
    }

    /// Handles a new connection by checking if it is a valid packet
    pub fn handle_new_connection(&self, mut stream: TcpStream) -> std::io::Result<()> {
        match Packet::from_bytes(&mut stream) {
            Ok(packet) => self.handle_incoming_packet(packet, stream),
            Err(err) => {
                self.log_file
                    .error(&format!("Error reading packet: {:?}", err));
                return Ok(());
            }
        }
        Ok(())
    }

    /// Initializes the task handler thread
    pub fn initialize_task_handler_thread(task_handler: TaskHandler) {
        std::thread::spawn(move || {
            task_handler.run();
        });
    }

    /// Handles an incoming packet from a connection. If it is a Connect packet, it will create a new client. Otherwise, it will log an error.
    pub fn handle_incoming_packet(&self, packet: Packet, stream: TcpStream) {
        match packet {
            Packet::Connect(connect_packet) => self.connect_new_client(connect_packet, stream),
            _ => self.log_file.error("Received an unsupported packet type"),
        }
    }

    /// Establishes a new connection with a client by creating a new client and a new thread for it
    pub fn connect_new_client(&self, connect_packet: Connect, stream: TcpStream) {
        let message = format!(
            "Received Connect Packet from client with ID: {}",
            connect_packet.client_id()
        );
        self.log_file.info(&message);

        let client_manager = self.client_manager.read().unwrap();

        let stream_clone = stream.try_clone().unwrap();

        match client_manager.process_connect_packet(connect_packet, stream_clone) {
            Some(new_client) => {
                self.log_file.info("Client connected successfully");

                let client_id = new_client.id();
                handle_connect(self.client_actions_sender.clone(), new_client);
                self.create_new_client_thread(
                    self.client_actions_sender.clone(),
                    stream,
                    client_id,
                    self.log_file.clone(),
                );
            }
            None => {
                self.log_file.error("Error connecting client");
            }
        };
    }

    /// Creates a new thread for a client
    pub fn create_new_client_thread(
        &self,
        sender_to_task_channel: std::sync::mpsc::Sender<Task>,
        mut stream: TcpStream,
        client_id: Vec<u8>,
        log_file: Arc<Logger>,
    ) {
        thread::spawn(move || {
            loop {
                let packet = Packet::from_bytes(&mut stream);
                match packet {
                    Ok(packet) => {
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
                        log_file.error(&format!("Connection Error: {:?}", err));
                        break;
                    }
                }
            }
            log_file.info("Closing connection");
        });
    }
}

/// Handles a packet by checking its type and calling the corresponding function
pub fn handle_packet(
    packet: Packet,
    client_id: Vec<u8>,
    stream: &mut TcpStream,
    sender_to_task_channel: std::sync::mpsc::Sender<Task>,
    log_file: Arc<Logger>,
) -> bool {
    let log_message = |packet_type: &str| {
        log_file.info(&format!(
            "Received {} packet from client: {}",
            packet_type,
            String::from_utf8_lossy(&client_id)
        ));
    };
    match packet {
        Packet::Publish(publish_packet) => {
            log_message("Publish");
            handle_publish(publish_packet, sender_to_task_channel, client_id)
        }
        Packet::Puback(puback_packet) => {
            log_message("Puback");
            handle_puback(puback_packet, sender_to_task_channel, client_id)
        }
        Packet::Subscribe(subscribe_packet) => {
            log_message("Subscribe");
            handle_subscribe(subscribe_packet, sender_to_task_channel, client_id)
        }
        Packet::Unsubscribe(unsubscribe_packet) => {
            log_message("Unsubscribe");
            handle_unsubscribe(unsubscribe_packet, sender_to_task_channel, client_id)
        }
        Packet::Pingreq(pingreq_packet) => {
            log_message("Pingreq");
            handle_pingreq(sender_to_task_channel, client_id)
        }
        Packet::Disconnect(disconnect_packet) => {
            log_message("Disconnect");
            disconnect_client(sender_to_task_channel, client_id)
        }
        _ => {
            log_file.error("Unsupported packet type");
            log_file.info("Disconnecting client");
            disconnect_client(sender_to_task_channel, client_id);
            false
        }
    }
}

/// Handles a CONNECT packet
pub fn handle_connect(
    sender_to_topics_channel: std::sync::mpsc::Sender<Task>,
    client: Client,
) -> bool {
    sender_to_topics_channel
        .send(Task::ConnectClient(client))
        .unwrap();
    true
}

/// Handles a PUBLISH packet
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

/// Handles a PUBACK packet
pub fn handle_puback(
    puback_packet: Puback,
    sender_to_topics_channel: std::sync::mpsc::Sender<Task>,
    client_id: Vec<u8>,
) -> bool {
    true
}

/// Handles a SUBSCRIBE packet
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

/// Handles an UNSUBSCRIBE packet
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

/// Handles a PINGREQ packet
pub fn handle_pingreq(
    sender_to_task_channel: std::sync::mpsc::Sender<Task>,
    client_id: Vec<u8>,
) -> bool {
    sender_to_task_channel
        .send(Task::RespondPing(client_id))
        .unwrap();
    true
}

/// Handles a DISCONNECT packet
pub fn disconnect_client(
    sender_to_task_channel: std::sync::mpsc::Sender<Task>,
    client_id: Vec<u8>,
) -> bool {
    sender_to_task_channel
        .send(Task::DisconnectClient(client_id))
        .unwrap();
    false
}
