use std::{
    net::{TcpListener, TcpStream},
    sync::{
        mpsc::{self, Sender},
        Arc, RwLock,
    },
    thread,
};

pub use mqtt::model::{
    packet::Packet,
    packets::{connect::Connect, publish::Publish, subscribe::Subscribe, unsubscribe::Unsubscribe},
};

use crate::{client::Client, client_manager::ClientManager};

use super::{
    config::Config,
    error::{ServerError, ServerResult},
    logfile::Logger,
    task_handler::{Task, TaskHandler},
};

/// Represents the MQTT server that will be handling all messages
/// The server has a configuration, a channel to send messages to clients, a log file, and a client manager
/// The server will be listening for incoming connections and handling them
/// It creates a new client for each connection and a new thread for each client
pub struct Server {
    /// Configuration of the server
    config: Config,
    /// Channel to send messages to clients
    client_actions_sender: Sender<Task>,
    /// Log file to log messages
    log_file: Arc<Logger>,
    /// Manages the registered clients in the server
    client_manager: Arc<RwLock<ClientManager>>,
}

impl Server {
    /// Creates a new server with the specified configuration
    /// Set up the client manager and the task handler thread
    /// Returns a ServerResult with the server if successful
    pub fn new(config: Config) -> ServerResult<Self> {
        let (client_actions_sender, client_actions_receiver) = mpsc::channel();

        let log_file = Arc::new(Logger::new(config.get_log_file()));
        let client_manager = ClientManager::new(config.get_login_file());
        let client_manager = Arc::new(RwLock::new(client_manager));

        let task_handler = TaskHandler::new(
            client_actions_receiver,
            log_file.clone(),
            client_manager.clone(),
            *config.get_key(),
        );

        task_handler.initialize_task_handler_thread();

        Ok(Server {
            config,
            client_actions_sender,
            log_file,
            client_manager,
        })
    }

    /// Starts the server
    pub fn server_run(&self) -> ServerResult<()> {
        let address = self.config.get_address();
        let key = self.config.get_key();

        self.log_file
            .info(&format!("Server running on address: {}\n", address));
        let listener = TcpListener::bind(address)?;

        for stream_result in listener.incoming() {
            match stream_result {
                Ok(stream) => {
                    self.log_file.info("New connection received");
                    self.handle_new_connection(stream, key)?;
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
    pub fn handle_new_connection(&self, mut stream: TcpStream, key: &[u8; 32]) -> ServerResult<()> {
        match Packet::from_bytes(&mut stream, key) {
            Ok(packet) => self.handle_incoming_packet(packet, stream)?,
            Err(err) => {
                self.log_file
                    .error(&format!("Error reading packet: {:?}", err));
            }
        }
        Ok(())
    }

    /// Handles an incoming packet from a connection. If it is a Connect packet, it will create a new client. Otherwise, it will log an error.
    pub fn handle_incoming_packet(&self, packet: Packet, stream: TcpStream) -> ServerResult<()> {
        match packet {
            Packet::Connect(connect_packet) => self.connect_new_client(connect_packet, stream),
            _ => {
                self.log_file.error("Received an unsupported packet type");
                Err(ServerError::UnsupportedPacket)
            }
        }
    }

    /// Establishes a new connection with a client by creating a new client and a new thread for it
    pub fn connect_new_client(
        &self,
        connect_packet: Connect,
        stream: TcpStream,
    ) -> ServerResult<()> {
        let message = format!(
            "Received Connect Packet from client with ID: {}",
            connect_packet.client_id()
        );
        self.log_file.info(&message);

        let client_manager = self.client_manager.read().map_err(|_| {
            ServerError::ClientConnection("Failed to acquire read lock".to_string())
        })?;

        let stream_clone = stream.try_clone()?;

        match client_manager.process_connect_packet(
            connect_packet,
            stream_clone,
            self.config.get_key(),
        ) {
            Some(new_client) => {
                self.log_file.info("Client connected successfully");

                let client_id = new_client.id();
                handle_connect(self.client_actions_sender.clone(), new_client)?;
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
        }
        Ok(())
    }

    /// Creates a new thread for a client
    pub fn create_new_client_thread(
        &self,
        sender_to_task_channel: std::sync::mpsc::Sender<Task>,
        mut stream: TcpStream,
        client_id: Vec<u8>,
        log_file: Arc<Logger>,
    ) {
        let key = *self.config.get_key();

        thread::spawn(move || {
            loop {
                let packet = Packet::from_bytes(&mut stream, &key);
                match packet {
                    Ok(packet) => {
                        if !handle_packet(
                            packet,
                            client_id.clone(),
                            sender_to_task_channel.clone(),
                            log_file.clone(),
                        ) {
                            break;
                        }
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
            handle_publish(publish_packet, sender_to_task_channel, client_id).unwrap_or(false)
        }
        Packet::Subscribe(subscribe_packet) => {
            log_message("Subscribe");
            handle_subscribe(subscribe_packet, sender_to_task_channel, client_id).unwrap_or(false)
        }
        Packet::Unsubscribe(unsubscribe_packet) => {
            log_message("Unsubscribe");
            handle_unsubscribe(unsubscribe_packet, sender_to_task_channel, client_id)
                .unwrap_or(false)
        }
        Packet::Pingreq(_) => {
            log_message("Pingreq");
            handle_pingreq(sender_to_task_channel, client_id).unwrap_or(false)
        }
        Packet::Disconnect(_) => {
            log_message("Disconnect");
            disconnect_client(sender_to_task_channel, client_id).unwrap_or(false)
        }
        _ => {
            log_file.error("Unsupported packet type");
            log_file.info("Disconnecting client");
            disconnect_client(sender_to_task_channel, client_id).unwrap_or(false);
            false
        }
    }
}

/// Handles a CONNECT packet
pub fn handle_connect(
    sender_to_topics_channel: std::sync::mpsc::Sender<Task>,
    client: Client,
) -> ServerResult<bool> {
    sender_to_topics_channel.send(Task::ConnectClient(client))?;
    Ok(true)
}

/// Handles a PUBLISH packet
pub fn handle_publish(
    publish_packet: Publish,
    sender_to_topics_channel: std::sync::mpsc::Sender<Task>,
    client_id: Vec<u8>,
) -> ServerResult<bool> {
    sender_to_topics_channel.send(Task::Publish(publish_packet, client_id))?;
    Ok(true)
}

/// Handles a SUBSCRIBE packet
pub fn handle_subscribe(
    subscribe_packet: Subscribe,
    sender_to_task_channel: std::sync::mpsc::Sender<Task>,
    client_id: Vec<u8>,
) -> ServerResult<bool> {
    sender_to_task_channel.send(Task::SubscribeClient(subscribe_packet, client_id))?;

    Ok(true)
}

/// Handles an UNSUBSCRIBE packet
pub fn handle_unsubscribe(
    unsubscribe_packet: Unsubscribe,
    sender_to_task_channel: std::sync::mpsc::Sender<Task>,
    client_id: Vec<u8>,
) -> ServerResult<bool> {
    sender_to_task_channel.send(Task::UnsubscribeClient(unsubscribe_packet, client_id))?;

    Ok(true)
}

/// Handles a PINGREQ packet
pub fn handle_pingreq(
    sender_to_task_channel: std::sync::mpsc::Sender<Task>,
    client_id: Vec<u8>,
) -> ServerResult<bool> {
    sender_to_task_channel.send(Task::RespondPing(client_id))?;
    Ok(true)
}

/// Handles a DISCONNECT packet
pub fn disconnect_client(
    sender_to_task_channel: std::sync::mpsc::Sender<Task>,
    client_id: Vec<u8>,
) -> ServerResult<bool> {
    sender_to_task_channel.send(Task::DisconnectClient(client_id))?;
    Ok(false)
}
