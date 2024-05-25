#![allow(dead_code)]
#![allow(unused_variables)]
use std::{
    collections::{HashMap, HashSet},
    io::Write,
    sync::{mpsc, RwLock},
    time::Duration,
};

use crate::client::Client;
use sauron::model::{
    components::{qos::QoS, topic_name::TopicName},
    packets::{
        connack::Connack, pingresp::Pingresp, puback::Puback, publish::Publish,
        subscribe::Subscribe, unsubscribe::Unsubscribe,
    },
    return_codes::connect_return_code::ConnectReturnCode,
};

pub enum Task {
    SubscribeClient(Subscribe, Vec<u8>),
    UnsubscribeClient(Unsubscribe, Vec<u8>),
    Publish(Publish, Vec<u8>),
    ConnectClient(Client),
    DisconnectClient(Vec<u8>),
    RespondPing(Vec<u8>),
}
#[derive(Clone, Debug)]
pub struct SubscriptionData {
    qos: QoS,
}
#[derive(Clone, Debug)]
pub struct Message {
    pub client_id: Vec<u8>,
    pub packet: Publish,
}

impl Message {
    pub fn new(client_id: Vec<u8>, packet: Publish) -> Self {
        Self { client_id, packet }
    }

    pub fn client_id(&self) -> &Vec<u8> {
        &self.client_id
    }

    pub fn packet(&self) -> &Publish {
        &self.packet
    }
}

type Subscribers = HashMap<Vec<u8>, SubscriptionData>; // key : client_id , value: SubscriptionData
                                                       // type Subtopic = HashMap<Vec<u8>, Topic>; // key: level, value: Topic
type Subscriptions = HashMap<TopicName, SubscriptionData>; // key: topic_name, value: SubscriptionData
type ClientId = Vec<u8>;

// #[derive(Debug)]
// pub struct Topic {
//     subscribers: RwLock<Subscribers>,
//     retained_messages: RwLock<Vec<Message>>,
//     subtopics: RwLock<Subtopic>,
//     subscriptions: RwLock<Subscriptions>,
// }

// impl Topic {
//     pub fn new() -> Self {
//         Topic {
//             subscribers: RwLock::new(HashMap::new()),
//             retained_messages: RwLock::new(Vec::new()),
//             subtopics: RwLock::new(HashMap::new()),
//             subscriptions: RwLock::new(HashMap::new()),
//         }
//     }

//     // todo: replace the unwraps
//     pub fn subscribe(
//         &self,
//         topic: &Topic,
//         mut levels: Vec<TopicLevel>,
//         client_id: Vec<u8>,
//         data: SubscriptionData,
//     ) {
//         if levels.is_empty() {
//             self.add_subscriber(client_id, data);
//             return;
//         }
//         let current_level = levels.remove(0);
//         let mut subtopics = self.subtopics.write().unwrap();
//         let subtopic = subtopics
//             .entry(current_level.to_bytes())
//             .or_insert(Topic::new());
//         subtopic.subscribe(subtopic, levels, client_id, data);
//     }

//     pub fn publish(
//         &self,
//         topic_name: TopicName,
//         message: Message,
//         clients: &HashMap<Vec<u8>, Client>,
//         active_connections: &HashSet<Vec<u8>>,
//     ) {
//         let subscribers = self.get_all_matching_subscriptions(topic_name);
//         for subscriber in subscribers {
//             for (client_id, data) in subscriber {
//                 // if !active_connections.contains(&client_id){
//                 //     clients.get(&client_id).unwrap().unreceived_messages.(message.packet.clone());
//                 //     continue;
//                 // }
//                 let client = clients.get(&client_id).unwrap();

//                 let publish_packet = message.packet.clone();
//                 if client
//                     .stream
//                     .lock()
//                     .unwrap()
//                     .write(publish_packet.to_bytes().as_slice())
//                     .is_ok()
//                 {};
//             }
//         }
//     }

//     pub fn get_all_matching_subscriptions(&self, topic_name: TopicName) -> Vec<Subscribers> {
//         let mut subscribers = Vec::new();
//         self.collect_matching_subscriptions(&mut subscribers, topic_name.levels);
//         subscribers
//     }

//     pub fn collect_matching_subscriptions(
//         &self,
//         subscribers: &mut Vec<Subscribers>,
//         levels: Vec<Vec<u8>>,
//     ) {
//         if levels.is_empty() {
//             subscribers.push(self.subscribers.read().unwrap().clone());
//             return;
//         }
//         let current_level = &levels[0];
//         let remaining_levels = levels[1..].to_vec();

//         let subtopics = self.subtopics.read().unwrap();
//         if let Some(subtopic) = subtopics.get(current_level) {
//             subtopic.collect_matching_subscriptions(subscribers, remaining_levels);
//         }
//     }

//     pub fn add_subscriber(&self, client_id: Vec<u8>, data: SubscriptionData) {
//         let mut subscribers = self.subscribers.write().unwrap();
//         subscribers.insert(client_id, data);
//     }

//     pub fn remove_subscriber(&self, client_id: Vec<u8>) {
//         let mut subscribers = self.subscribers.write().unwrap();
//         subscribers.remove(&client_id);
//     }

//     pub fn add_retained_message(&self, message: Message) {
//         let mut retained_messages = self.retained_messages.write().unwrap();
//         retained_messages.push(message);
//     }
// }

#[derive(Debug)]
pub struct TaskHandler {
    client_actions_receiver_channel: mpsc::Receiver<Task>,
    clients: RwLock<HashMap<Vec<u8>, Client>>,
    active_connections: RwLock<HashSet<Vec<u8>>>,
    retained_messages: RwLock<HashMap<Vec<u8>, Publish>>,
    topics: RwLock<HashMap<TopicName, Vec<ClientId>>>,
}

impl TaskHandler {
    pub fn new(receiver_channel: mpsc::Receiver<Task>) -> Self {
        TaskHandler {
            client_actions_receiver_channel: receiver_channel,
            clients: RwLock::new(HashMap::new()),
            active_connections: RwLock::new(HashSet::new()),
            retained_messages: RwLock::new(HashMap::new()),
            topics: RwLock::new(HashMap::new()),
        }
    }

    pub fn initialize_task_handler_thread(self) {
        println!("Starting task handler thread\n");
        std::thread::spawn(move || {
            self.run();
        });
    }

    pub fn run(mut self) {
        loop {
            match self.client_actions_receiver_channel.recv() {
                Ok(task) => match task {
                    Task::SubscribeClient(subscribe, client_id) => {
                        println!(
                            "Task Handler received task: subscribe Client: {:?}\n",
                            std::str::from_utf8(&client_id).unwrap()
                        );
                        self.subscribe(subscribe, client_id);
                    }
                    Task::UnsubscribeClient(unsubscribe, client_id) => {
                        println!(
                            "Task Handler received task: unsubscribe Client: {:?}\n",
                            client_id
                        );
                        self.unsubscribe(unsubscribe);
                    }
                    Task::Publish(publish, client_id) => {
                        println!(
                            "Task Handler received task: Publish message: {:?}\n",
                            publish
                        );
                        self.publish(&publish, client_id);
                    }
                    Task::ConnectClient(client) => {
                        println!("Task Handler received task: Client Connected");
                        self.handle_new_client_connection(client);
                    }
                    Task::DisconnectClient(client_id) => {
                        self.handle_client_disconnected(client_id);
                    }
                    Task::RespondPing(client_id) => {
                        println!("Task Handler received task: Respond Ping");
                        self.respond_ping(client_id);
                    }
                },
                Err(_) => {
                    std::thread::sleep(Duration::from_secs(1));
                    continue;
                }
            }
        }
    }

    // Subscribe a client_id into a set of topics given a Subscribe packet

    pub fn subscribe(&self, subscribe_packet: Subscribe, client_id: Vec<u8>) {
        let mut clients = self.clients.write().unwrap();

        // cambio para obtener una referencia mutable del client y mutarlo agregandole la subscripcion
        // al cliente ademas de hacerlo en el task handler.
        if let Some(client) = clients.get_mut(&client_id) {
            for (topic_filter, qos) in subscribe_packet.topics() {
                let mut levels: Vec<Vec<u8>> = vec![];
                for level in topic_filter.levels() {
                    levels.push(level.to_bytes());
                }
                let topic_name = TopicName::new(levels, false);

                client.add_subscription(topic_name.clone());

                let mut topics = self.topics.write().unwrap();
                topics
                    .entry(topic_name.clone())
                    .or_default()
                    .push(client_id.clone());
            }
            println!("Active clients: {:?}\n", clients);
            println!("Active topics with subscribers: {:?}\n", self.topics);
        } else {
            println!("Client does not exist");
        }
    }

    // Unsubscribe a client_id from a set of topics given an Unsubscribe packet
    pub fn unsubscribe(&self, unsubscribe_packet: Unsubscribe) {
        todo!()
    }

    /*publish uses a publish method of the topic struct and also sends to the clients subscribed to the topic the message*/
    pub fn publish(&self, publish_packet: &Publish, client_id: Vec<u8>) {
        let topic_name = publish_packet.topic();

        let binding = self.topics.read().unwrap();
        let clients = match binding.get(topic_name) {
            Some(clients) => clients,
            None => {
                println!("No clients subscribed to topic: {:?}", topic_name);
                return;
            }
        };

        let message = Message::new(client_id.clone(), publish_packet.clone());

        for client_id in clients {
            if let Some(client) = self.clients.read().unwrap().get(client_id) {
                client.send_message(message.clone());
            }
        }

        //self.puback(publish_packet.package_identifier(), client_id.clone());
    }

    pub fn handle_new_client_connection(&self, client: Client) {
        let connack_packet = Connack::new(true, ConnectReturnCode::ConnectionAccepted);
        let connack_packet_vec = connack_packet.to_bytes();
        let connack_packet_bytes = connack_packet_vec.as_slice();

        let client_id = client.id.clone();

        let mut clients = self.clients.write().unwrap();
        let active_connections = self.active_connections.write().unwrap();
        //insert the client id as key and the client as value in clients in the rwlockwriteguard
        clients.entry(client_id.clone()).or_insert(client);

        let mut stream = match clients.get(&client_id).unwrap().stream.lock() {
            Ok(stream) => stream,
            Err(_) => {
                println!("Error getting new client's stream. Connection will not be accepted.");
                return;
            }
        };

        match stream.write(connack_packet_bytes) {
            Ok(_) => {
                drop(active_connections);
                println!("New client connected! ID number: {:?}", client_id);
            }
            Err(_) => {
                println!("Error sending ping response to client: {:?}", client_id);
            }
        };
    }

    pub fn puback(&self, package_identifier: Option<u16>, client_id: Vec<u8>) {
        let puback_packet = Puback::new(package_identifier);
        let puback_packet_vec = puback_packet.to_bytes();
        let puback_packet_bytes = puback_packet_vec.as_slice();

        let clients = self.clients.write().unwrap();

        let client = match clients.get(&client_id) {
            Some(client) => client,
            None => {
                println!("Error: client not found in client list!");
                return;
            }
        };

        let mut stream = match client.stream.lock() {
            Ok(stream) => stream,
            Err(_) => {
                println!("Error getting stream, puback will not be sent to client.");
                return;
            }
        };

        match stream.write(puback_packet_bytes) {
            Ok(_) => {
                println!("Puback sent to client: {:?}", client_id);
            }
            Err(_) => {
                println!("Error sending puback to client: {:?}", client_id);
            }
        };
    }

    pub fn respond_ping(&self, client_id: Vec<u8>) {
        let clients = self.clients.write().unwrap();

        let client = clients.get(&client_id).unwrap();
        let pingresp_packet = Pingresp::new();
        let pingresp_packet_vec = pingresp_packet.to_bytes();
        let pingresp_packet_bytes = pingresp_packet_vec.as_slice();

        let mut stream = match client.stream.lock() {
            Ok(stream) => stream,
            Err(_) => {
                println!("Error getting stream, ping will not be responded to client");
                return;
            }
        };

        match stream.write(pingresp_packet_bytes) {
            Ok(_) => {
                println!("Ping response sent to client: {:?}", client_id);
            }
            Err(_) => {
                println!("Error sending ping response to client: {:?}", client_id);
            }
        };
    }

    pub fn handle_client_disconnected(&mut self, client_id: Vec<u8>) {
        //self.active_connections.remove(&client_id);
        let mut active_connections = self.active_connections.write().unwrap();
        active_connections.remove(&client_id);
    }

    pub fn register_puback(&mut self, puback: Puback) {
        let message_id = match puback.packet_identifier() {
            Some(id) => id,
            None => {
                println!("Error: Puback packet does not have a packet identifier.");
                return;
            }
        };
        let message_id_bytes = message_id.to_be_bytes().to_vec();

        let mut retained_messages = self.retained_messages.write().unwrap();

        retained_messages.remove(&message_id_bytes);
    }
}
