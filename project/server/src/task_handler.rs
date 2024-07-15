use std::{
    collections::{HashMap, HashSet, VecDeque},
    io::{Read, Write},
    sync::{mpsc, Arc, RwLock},
    thread,
    time::{Duration, Instant},
};

use crate::{
    client::Client, client_manager::ClientManager, config::Config, error::ServerResult,
    logfile::Logger,
};

use mqtt::model::packet::Packet;

use mqtt::model::{
    components::{qos::QoS, topic_filter::TopicFilter, topic_name::TopicName},
    packets::{
        connack::Connack, pingresp::Pingresp, puback::Puback, publish::Publish, suback::Suback,
        subscribe::Subscribe, unsuback::Unsuback, unsubscribe::Unsubscribe,
    },
    return_codes::{connect_return_code::ConnectReturnCode, suback_return_code::SubackReturnCode},
};

use std::fs::File;

/// Represents the different tasks that the task handler can perform
pub enum Task {
    SubscribeClient(Subscribe, Vec<u8>),
    UnsubscribeClient(Unsubscribe, Vec<u8>),
    Publish(Publish, Vec<u8>),
    ConnectClient(Client),
    DisconnectClient(Vec<u8>),
    RespondPing(Vec<u8>),
}

const ADMIN_ID: &[u8] = b"admin";
const CLIENT_REGISTER: &[u8] = b"$client-register";
const SEPARATOR: u8 = b';';

const RETAINED_MESSAGES_TAG: &str = "R";
const OFFLINE_MESSAGES_TAG: &str = "O";
const CLIENTS_TAG: &str = "C";

/// Represents the task handler that will handle all the tasks that the server needs to process
#[derive(Debug)]
pub struct TaskHandler {
    client_actions_receiver_channel: mpsc::Receiver<Task>,
    clients: RwLock<HashMap<Vec<u8>, Client>>,
    active_connections: HashSet<Vec<u8>>,
    offline_messages: HashMap<Vec<u8>, VecDeque<Publish>>,
    retained_messages: HashMap<TopicName, VecDeque<Publish>>,
    log_file: Arc<Logger>,
    client_manager: Arc<RwLock<ClientManager>>,
    key: [u8; 32],
    backup_file: Option<String>,
    segs_to_backup: u32,
}

impl TaskHandler {
    /// Creates a new task handler with the specified receiver channel, logger and client manager
    pub fn default(
        receiver_channel: mpsc::Receiver<Task>,
        log_file: Arc<Logger>,
        client_manager: Arc<RwLock<ClientManager>>,
        key: [u8; 32],
        segs_to_backup: u32,
        backup_file: Option<String>,
    ) -> Self {
        TaskHandler {
            client_actions_receiver_channel: receiver_channel,
            clients: RwLock::new(HashMap::new()),
            active_connections: HashSet::new(),
            offline_messages: HashMap::new(),
            retained_messages: HashMap::new(),
            log_file,
            client_manager,
            key,
            backup_file,
            segs_to_backup,
        }
    }

    pub fn new(
        client_actions_receiver_channel: mpsc::Receiver<Task>,
        config: &Config,
        client_manager: Arc<RwLock<ClientManager>>,
        log_file: Arc<Logger>,
    ) -> Self {
        let backup_file = config.get_backup_file();
        let initialize_with_backup = config.get_initialize_with_backup();
        let key = *config.get_key();
        let segs_to_backup = config.get_segs_to_backup();

        if !initialize_with_backup {
            log_file.info("Initializing server without backup");
            return TaskHandler::default(
                client_actions_receiver_channel,
                log_file,
                client_manager,
                key,
                segs_to_backup,
                backup_file,
            );
        }

        match backup_file {
            Some(backup_file_string) => match File::open(backup_file_string.clone()) {
                Ok(mut file) => {
                    let mut data = String::new();
                    if file.read_to_string(&mut data).is_err() {
                        log_file
                            .info("Error reading backup file. Initializing server without backup");
                        return TaskHandler::default(
                            client_actions_receiver_channel,
                            log_file,
                            client_manager,
                            key,
                            segs_to_backup,
                            Some(backup_file_string.clone()),
                        );
                    }

                    log_file.info("Initializing server with backup");
                    TaskHandler::deserialize(
                        &data,
                        client_manager.clone(),
                        log_file.clone(),
                        config,
                        client_actions_receiver_channel,
                    )
                }
                Err(_) => {
                    log_file.info("Error opening backup file. Initializing server without backup");
                    TaskHandler::default(
                        client_actions_receiver_channel,
                        log_file,
                        client_manager,
                        key,
                        segs_to_backup,
                        Some(backup_file_string.clone()),
                    )
                }
            },
            None => TaskHandler::default(
                client_actions_receiver_channel,
                log_file,
                client_manager,
                key,
                segs_to_backup,
                backup_file,
            ),
        }
    }

    /// Initializes the task handler thread
    pub fn initialize_task_handler_thread(self) {
        std::thread::spawn(move || {
            self.run();
        });
    }

    /// Runs the task handler in a loop
    pub fn run(mut self) {
        let backup_interval = Duration::from_secs(self.segs_to_backup as u64);
        let mut last_backup = std::time::Instant::now();

        loop {
            match self.client_actions_receiver_channel.recv() {
                Ok(task) => {
                    if let Err(e) = self.handle_task(task) {
                        self.log_file.error(e.to_string().as_str());
                    }
                }
                Err(_) => {
                    std::thread::sleep(Duration::from_secs(1));
                    continue;
                }
            }

            if self.backup_file.is_some() && last_backup.elapsed() >= backup_interval {
                self.log_file.info("Backing up server data");
                self.backup_data();
                last_backup = Instant::now();
            }
        }
    }

    /// Handles all possible tasks that the server can receive
    fn handle_task(&mut self, task: Task) -> ServerResult<()> {
        match task {
            Task::SubscribeClient(subscribe, client_id) => self.subscribe(subscribe, client_id),
            Task::UnsubscribeClient(unsubscribe, client_id) => {
                self.unsubscribe(unsubscribe, client_id)
            }
            Task::Publish(publish, client_id) => self.publish(&publish, client_id),
            Task::ConnectClient(client) => self.handle_new_client_connection(client),
            Task::DisconnectClient(client_id) => self.handle_client_disconnected(client_id),
            Task::RespondPing(client_id) => self.respond_ping(client_id),
        }
    }

    /// Subscribe a client_id into a set of topics given a Subscribe packet
    pub fn subscribe(&self, subscribe_packet: Subscribe, client_id: Vec<u8>) -> ServerResult<()> {
        let mut clients = self.clients.write()?;

        if let Some(client) = clients.get_mut(&client_id) {
            self.suback(subscribe_packet.packet_identifier(), client);

            self.log_file
                .log_successful_subscription(&client_id, &subscribe_packet);

            for (topic_filter, _) in subscribe_packet.topics() {
                client.add_subscription(topic_filter.clone());

                // Send the retained message if it exists
                for (topic_name, retained_messages) in &self.retained_messages {
                    if topic_filter.match_topic_name(topic_name.clone()) {
                        for message in retained_messages {
                            client.send_message(message.clone(), &self.log_file, &self.key);
                        }
                    }
                }
            }
        } else {
            self.log_file.log_client_does_not_exist(&client_id);
        }
        Ok(())
    }

    /// Unsubscribe a client_id from a set of topics given an Unsubscribe packet
    pub fn unsubscribe(
        &self,
        unsubscribe_packet: Unsubscribe,
        client_id: Vec<u8>,
    ) -> ServerResult<()> {
        let mut clients = self.clients.write()?;

        if let Some(client) = clients.get_mut(&client_id) {
            for topic_filter in unsubscribe_packet.topics() {
                client.remove_subscription(topic_filter);
            }

            self.log_file
                .log_successful_unsubscription(&client_id, &unsubscribe_packet);
            self.unsuback(unsubscribe_packet.packet_identifier(), client);
        } else {
            self.log_file.log_client_does_not_exist(&client_id);
        }

        Ok(())
    }

    /// Publish a message to all clients subscribed to the topic of the Publish packet
    pub fn publish(&mut self, publish_packet: &Publish, client_id: Vec<u8>) -> ServerResult<()> {
        let topic_name = publish_packet.topic();

        if topic_name.server_reserved() {
            self.handle_server_reserved_topic(publish_packet, client_id);
            return Ok(());
        }

        if publish_packet.retain() {
            self.retained_messages
                .entry(topic_name.clone())
                .or_default()
                .push_back(publish_packet.clone());
        }

        let mut clients = vec![];

        for client in self.clients.read()?.values() {
            if client.is_subscribed(topic_name) {
                clients.push(client.id());
            }
        }

        if clients.is_empty() {
            let message = format!("No clients subscribed to topic: {}", topic_name);
            self.log_file.error(message.as_str());
            return Ok(());
        }

        self.log_file
            .log_successful_publish(&client_id, publish_packet);

        for client_id in clients {
            if let Some(client) = self.clients.read()?.get(&client_id) {
                if self.active_connections.contains(&client_id) {
                    client.send_message(publish_packet.clone(), &self.log_file, &self.key);
                } else {
                    self.offline_messages
                        .entry(client_id.clone())
                        .or_default()
                        .push_back(publish_packet.clone());
                }
            }
        }

        let mut clients = self.clients.write()?;

        // If QoS is not AtMostOnce, send a Puback packet to the client that published the message
        if &QoS::AtMost != publish_packet.qos() {
            if let Some(client) = clients.get_mut(&client_id) {
                self.puback(publish_packet.package_identifier(), client);
            }
        }

        let clients_retained_messages = self.offline_messages.get(&client_id);
        let client = match clients.get_mut(&client_id) {
            Some(client) => client,
            None => {
                self.log_file.log_client_does_not_exist(&client_id);
                return Ok(());
            }
        };

        if let Some(clients_retained_messages) = clients_retained_messages {
            self.handle_retained_messages(client, clients_retained_messages);
            match self.offline_messages.get_mut(&client_id) {
                Some(queue) => queue.clear(),
                None => {
                    self.log_file.error("Error clearing offline messages");
                }
            }
        }

        Ok(())
    }

    /// Handle a server reserved topic (e.g. $client-register)
    pub fn handle_server_reserved_topic(&self, publish_packet: &Publish, client_id: Vec<u8>) {
        let topic_name = publish_packet.topic();
        let levels = topic_name.levels();

        if client_id != ADMIN_ID {
            self.log_file.error("Client is not admin");
            return;
        }

        if levels.len() == 1 && levels[0] == CLIENT_REGISTER {
            let message = publish_packet.message();
            //  split username and password by SEPARATOR
            let split = message.split(|&c| c == SEPARATOR).collect::<Vec<&[u8]>>();

            if split.len() != 3 {
                self.log_file
                    .error("Invalid message for client registration");
                return;
            }

            let client_id = split[0].to_vec();
            let username = split[1].to_vec();
            let password = split[2].to_vec();

            let client_manager = self.client_manager.write().unwrap();
            
            match client_manager.authenticate_client(client_id.clone(), username.clone(), password.clone()) {
                Ok(true) => {
                    self.log_file.info("Client already registered");
                },
                Ok(false) => {
                    self.log_file.log_client_registrated(&client_id.clone());
                    let _ = client_manager.register_client(client_id, username, password);
                },
                Err(e) => {
                    self.log_file.error(e.to_string().as_str());
                }
            }
        } else {
            self.log_file
                .error("Invalid topic for server reserved topic");
        }
    }

    /// Handle retained messages for a client
    pub fn handle_retained_messages(
        &self,
        client: &mut Client,
        retained_messages: &VecDeque<Publish>,
    ) {
        for message in retained_messages {
            client.send_message(message.clone(), &self.log_file, &self.key);
        }
    }

    /// Handle a new client connection
    pub fn handle_new_client_connection(&mut self, client: Client) -> ServerResult<()> {
        let connack_packet = Connack::new(true, ConnectReturnCode::ConnectionAccepted);
        let connack_packet_vec = connack_packet.to_bytes(&self.key);
        let connack_packet_bytes = connack_packet_vec.as_slice();

        let client_id = client.id();
        let mut clients = self.clients.write()?;

        if clients.contains_key(&client_id) {
            let message = format!("Client {} reconnected", String::from_utf8_lossy(&client_id));
            self.log_file.info(message.as_str());
            let old_client = match clients.get_mut(&client_id) {
                Some(client) => client,
                None => {
                    self.log_file.error("Error retreiving the old client for reconnection. Connection will not be accepted.");
                    return Ok(());
                }
            };
            old_client.stream = client.stream;
        } else {
            clients.entry(client_id.clone()).or_insert(client);
        }

        let client = match clients.get(&client_id) {
            Some(client) => client,
            None => {
                self.log_file.log_client_does_not_exist(&client_id);
                return Ok(());
            }
        };

        let mut stream = match &client.stream {
            Some(stream) => stream,
            None => {
                self.log_file
                    .log_error_getting_stream(&client_id, "Connack");
                return Ok(());
            }
        };

        match stream.write_all(connack_packet_bytes) {
            Ok(_) => {
                self.active_connections.insert(client_id.clone());
                let message = format!(
                    "New client connected! ID: {:?}",
                    String::from_utf8_lossy(&client_id)
                );
                self.log_file.info(message.as_str());
                self.log_file.log_info_sent_packet("Connack", &client_id);
            }
            Err(_) => self
                .log_file
                .log_error_sending_packet("Connack", &client_id),
        };
        Ok(())
    }

    /// Send a suback packet to a client
    pub fn suback(&self, package_identifier: u16, client: &mut Client) {
        let suback_packet = Suback::new(
            package_identifier,
            vec![SubackReturnCode::SuccessMaximumQoS0],
        );
        let suback_packet_vec = suback_packet.to_bytes(&self.key);
        let suback_packet_bytes = suback_packet_vec.as_slice();

        let mut stream = match &client.stream {
            Some(stream) => stream,
            None => {
                self.log_file
                    .log_error_getting_stream(&client.id, "Connack");
                return;
            }
        };

        match stream.write_all(suback_packet_bytes) {
            Ok(_) => self.log_file.log_info_sent_packet("Suback", &client.id()),
            Err(_) => self
                .log_file
                .log_error_sending_packet("Suback", &client.id()),
        };
    }

    /// Send a puback packet to a client
    pub fn puback(&self, package_identifier: Option<u16>, client: &mut Client) {
        let puback_packet = Puback::new(package_identifier);
        let puback_packet_vec = puback_packet.to_bytes(&self.key);
        let puback_packet_bytes = puback_packet_vec.as_slice();

        let mut stream = match &client.stream {
            Some(stream) => stream,
            None => {
                self.log_file
                    .log_error_getting_stream(&client.id, "Connack");
                return;
            }
        };

        match stream.write_all(puback_packet_bytes) {
            Ok(_) => self.log_file.log_info_sent_packet("Puback", &client.id()),
            Err(_) => self
                .log_file
                .log_error_sending_packet("Puback", &client.id()),
        };
    }

    /// Send an unsuback packet to a client
    pub fn unsuback(&self, package_identifier: u16, client: &mut Client) {
        let unsuback_packet = Unsuback::new(package_identifier);
        let unsuback_packet_vec = unsuback_packet.to_bytes(&self.key);
        let unsuback_packet_bytes = unsuback_packet_vec.as_slice();

        let mut stream = match &client.stream {
            Some(stream) => stream,
            None => {
                self.log_file
                    .log_error_getting_stream(&client.id, "Connack");
                return;
            }
        };

        match stream.write_all(unsuback_packet_bytes) {
            Ok(_) => self.log_file.log_info_sent_packet("Unsuback", &client.id()),
            Err(_) => self
                .log_file
                .log_error_sending_packet("Unsuback", &client.id()),
        };
    }

    /// Send a ping response to a client
    pub fn respond_ping(&self, client_id: Vec<u8>) -> ServerResult<()> {
        let clients = self.clients.read()?;

        let client = match clients.get(&client_id) {
            Some(client) => client,
            None => {
                self.log_file.log_client_does_not_exist(&client_id);
                return Ok(());
            }
        };
        let pingresp_packet = Pingresp::new();
        let pingresp_packet_vec = pingresp_packet.to_bytes(&self.key);
        let pingresp_packet_bytes = pingresp_packet_vec.as_slice();

        let mut stream = match &client.stream {
            Some(stream) => stream,
            None => {
                self.log_file
                    .log_error_getting_stream(&client.id, "Connack");
                return Ok(());
            }
        };

        match stream.write_all(pingresp_packet_bytes) {
            Ok(_) => {
                self.log_file
                    .log_info_sent_packet("Ping response", &client_id);
            }
            Err(_) => {
                self.log_file
                    .log_error_sending_packet("Ping response", &client_id);
            }
        };
        Ok(())
    }

    /// Handle a client disconnection
    pub fn handle_client_disconnected(&mut self, client_id: Vec<u8>) -> ServerResult<()> {
        self.active_connections.remove(&client_id);
        self.client_manager
            .write()?
            .disconnect_client(client_id.clone())?;
        Ok(())
    }

    /// Serialize the task handler data to a string
    fn serialize(&self) -> String {
        let mut serialized_data = String::new();

        // Serialize offline_messages
        for (client, queue) in &self.offline_messages {
            for publish in queue {
                serialized_data.push_str(&format!(
                    "{};{};{}\n",
                    OFFLINE_MESSAGES_TAG,
                    bytes_to_hex(client),
                    bytes_to_hex(&publish.to_bytes(&self.key))
                ));
            }
        }

        // Serialize retained_messages
        for (topic_name, messages) in &self.retained_messages {
            for message in messages {
                serialized_data.push_str(&format!(
                    "{};{};{}\n",
                    RETAINED_MESSAGES_TAG,
                    bytes_to_hex(&topic_name.to_bytes()),
                    bytes_to_hex(&message.to_bytes(&self.key))
                ));
            }
        }

        // Serialize clients
        let clients_read = match self.clients.read() {
            Ok(clients) => clients,
            Err(_) => {
                self.log_file
                    .error("Error reading clients for serialization");
                return serialized_data;
            }
        };

        for (id, client) in clients_read.iter() {
            for sub in &client.subscriptions {
                serialized_data.push_str(&format!(
                    "{};{};{}\n",
                    CLIENTS_TAG,
                    bytes_to_hex(id),
                    bytes_to_hex(&sub.to_bytes())
                ));
            }
        }

        serialized_data
    }

    /// Builds a TaskHandler from a backup file string
    fn deserialize(
        serialized_data: &str,
        client_manager: Arc<RwLock<ClientManager>>,
        log_file: Arc<Logger>,
        config: &Config,
        receiver_channel: mpsc::Receiver<Task>,
    ) -> TaskHandler {
        let key = *config.get_key();
        let mut offline_messages = HashMap::new();
        let mut retained_messages = HashMap::new();
        let mut clients = HashMap::new();

        for line in serialized_data.lines() {
            let parts: Vec<&str> = line.split(';').collect();
            if parts.len() != 3 {
                continue;
            }

            let record_type = parts[0];
            let entry_key = match hex_to_bytes(parts[1]) {
                Ok(k) => k,
                Err(_) => continue,
            };
            let value = match hex_to_bytes(parts[2]) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let mut value_stream = std::io::Cursor::new(value);

            match record_type {
                OFFLINE_MESSAGES_TAG => {
                    let publish = match Packet::from_bytes(&mut value_stream, &key) {
                        Ok(Packet::Publish(publish)) => publish,
                        _ => continue,
                    };
                    offline_messages
                        .entry(entry_key)
                        .or_insert_with(VecDeque::new)
                        .push_back(publish);
                }
                RETAINED_MESSAGES_TAG => {
                    let mut entry_stream = std::io::Cursor::new(entry_key);

                    let topic_name = TopicName::from_bytes(&mut entry_stream);
                    let topic_name = match topic_name {
                        Ok(topic_name) => topic_name,
                        Err(_) => {
                            println!("Error deserializing topic name");
                            continue;
                        }
                    };
                    let message = match Packet::from_bytes(&mut value_stream, &key) {
                        Ok(Packet::Publish(publish)) => publish,
                        _ => continue,
                    };
                    retained_messages
                        .entry(topic_name)
                        .or_insert_with(VecDeque::new)
                        .push_back(message);
                }
                CLIENTS_TAG => {
                    let subscription = TopicFilter::from_bytes(&mut value_stream);
                    let subscription = match subscription {
                        Ok(subscription) => subscription,
                        Err(_) => {
                            println!("Error deserializing subscription");
                            continue;
                        }
                    };
                    clients
                        .entry(entry_key.clone())
                        .or_insert_with(|| Client::new_from_backup(entry_key.clone(), Vec::new()))
                        .subscriptions
                        .push(subscription);
                }
                _ => {}
            }
        }

        let clients_lock = RwLock::new(clients);

        TaskHandler {
            client_actions_receiver_channel: receiver_channel,
            clients: clients_lock,
            active_connections: HashSet::new(),
            offline_messages,
            retained_messages,
            log_file,
            client_manager,
            key,
            backup_file: config.get_backup_file(),
            segs_to_backup: config.get_segs_to_backup(),
        }
    }

    pub fn backup_data(&self) {
        let backup_file_path_copy = match &self.backup_file {
            Some(file) => file.clone(),
            None => return,
        };
        let serialized_data = self.serialize();

        // Spawn a thread to serialize and write the data to a file as I/O operations are blocking
        thread::spawn(move || {
            let mut file = match File::create(backup_file_path_copy) {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("Failed to create/open backup file: {}", e);
                    return;
                }
            };
            if let Err(e) = file.write_all(serialized_data.as_bytes()) {
                eprintln!("Failed to write to backup file: {}", e);
            }
        });
    }
}

/// Convert a slice of bytes to a hexadecimal string
fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join("")
}

/// Convert a hexadecimal string to a vector of bytes
fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, String> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).map_err(|e| e.to_string()))
        .collect()
}
