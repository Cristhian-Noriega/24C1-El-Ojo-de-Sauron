use std::{
    collections::{HashMap, HashSet, VecDeque},
    io::Write,
    sync::{mpsc, Arc, RwLock},
    time::Duration,
};

use crate::{client::Client, client_manager::ClientManager, error::ServerResult, logfile::Logger};

use mqtt::model::{
    components::qos::QoS,
    packets::{
        connack::Connack, pingresp::Pingresp, puback::Puback, publish::Publish, suback::Suback,
        subscribe::Subscribe, unsuback::Unsuback, unsubscribe::Unsubscribe,
    },
    return_codes::{connect_return_code::ConnectReturnCode, suback_return_code::SubackReturnCode},
};

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

/// Represents the task handler that will handle all the tasks that the server needs to process
#[derive(Debug)]
pub struct TaskHandler {
    client_actions_receiver_channel: mpsc::Receiver<Task>,
    clients: RwLock<HashMap<Vec<u8>, Client>>,
    active_connections: HashSet<Vec<u8>>,
    retained_messages: HashMap<Vec<u8>, VecDeque<Publish>>,

    // monitor se subscribe a (drone-data/+) <- es un topic filter

    // drone data publica en (drone-data/1) <- esto es un topic name

    // debería guardar que monitor está suscrito a (drone-data/+)
    // cuando drone data publica en (drone-data/1) recorro todos los subscribes y me fijo si alguno matchea
    // si matchea, le mando el msg
    log_file: Arc<Logger>,
    client_manager: Arc<RwLock<ClientManager>>,
    key: [u8; 32],
}

impl TaskHandler {
    /// Creates a new task handler with the specified receiver channel, logger and client manager
    pub fn new(
        receiver_channel: mpsc::Receiver<Task>,
        log_file: Arc<Logger>,
        client_manager: Arc<RwLock<ClientManager>>,
        key: [u8; 32],
    ) -> Self {
        TaskHandler {
            client_actions_receiver_channel: receiver_channel,
            clients: RwLock::new(HashMap::new()),
            active_connections: HashSet::new(),
            retained_messages: HashMap::new(),
            log_file,
            client_manager,
            key,
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
            for (topic_filter, _) in subscribe_packet.topics() {
                client.add_subscription(topic_filter.clone());
            }
            self.log_file
                .log_successful_subscription(&client_id, &subscribe_packet);
            self.suback(subscribe_packet.packet_identifier(), client);
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
            if let Some(client) = self.clients.read().unwrap().get(&client_id) {
                if self.active_connections.contains(&client_id) {
                    client.send_message(publish_packet.clone(), &self.log_file, &self.key);
                } else {
                    self.retained_messages
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

        let clients_retained_messages = self.retained_messages.get(&client_id);
        let client = clients.get_mut(&client_id).unwrap();
        if let Some(clients_retained_messages) = clients_retained_messages {
            self.handle_retained_messages(client, clients_retained_messages);
            self.retained_messages.get_mut(&client_id).unwrap().clear();
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
            if client_manager.authenticate_client(
                client_id.clone(),
                username.clone(),
                password.clone(),
            ) {
                self.log_file.info("Client already registered");
            } else {
                self.log_file.log_client_registrated(&client_id.clone());
                client_manager.register_client(client_id, username, password);
            }
        } else {
            self.log_file
                .error("Invalid topic for server reserved topic");
        }
    }

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

        let mut stream = match client.stream.lock() {
            Ok(stream) => stream,
            Err(_) => {
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
        //return code hardcodeado
        let suback_packet = Suback::new(
            package_identifier,
            vec![SubackReturnCode::SuccessMaximumQoS0],
        );
        let suback_packet_vec = suback_packet.to_bytes(&self.key);
        let suback_packet_bytes = suback_packet_vec.as_slice();

        let mut stream = match client.stream.lock() {
            Ok(stream) => stream,
            Err(_) => {
                self.log_file
                    .log_error_getting_stream(&client.id(), "suback");
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

        let mut stream = match client.stream.lock() {
            Ok(stream) => stream,
            Err(_) => {
                self.log_file
                    .log_error_getting_stream(&client.id(), "puback");
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

        let mut stream = match client.stream.lock() {
            Ok(stream) => stream,
            Err(_) => {
                self.log_file
                    .log_error_getting_stream(&client.id(), "Unsuback");
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

        let mut stream = match client.stream.lock() {
            Ok(stream) => stream,
            Err(_) => {
                self.log_file
                    .log_error_getting_stream(&client_id, "Ping response");
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
        Ok(())
    }
}
