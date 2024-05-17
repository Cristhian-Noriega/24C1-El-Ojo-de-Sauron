use std::{
    collections::{HashMap, HashSet},
    sync::{mpsc, RwLock},
    time::Duration,
};

use crate::client::Client;
use sauron::model::{
    packets::{puback::Puback, publish::Publish, subscribe::Subscribe, unsubscribe::Unsubscribe},
    qos::QoS,
    topic_level::TopicLevel,
};

pub enum TopicHandlerTask {
    SubscribeClient(Subscribe, Vec<u8>),
    UnsubscribeClient(Unsubscribe, Vec<u8>),
    Publish(Publish, Vec<u8>),
    RegisterPubAck(Puback),
    ClientConnected(Client),
    ClientDisconnected(Vec<u8>),
}

pub struct SubscriptionData {
    qos: QoS,
}

pub struct Message {
    pub client_id: Vec<u8>,
    pub packet: Publish,
}

type Suscriber = HashMap<Vec<u8>, SubscriptionData>;
type Subtopic = HashMap<String, Topic>;

pub struct Topic {
    subscribers: RwLock<Suscriber>,
    retained_messages: RwLock<Vec<Message>>,
    subtopics: RwLock<Subtopic>,
}

impl Topic {
    pub fn new() -> Self {
        Topic {
            subscribers: RwLock::new(HashMap::new()),
            retained_messages: RwLock::new(Vec::new()),
            subtopics: RwLock::new(HashMap::new()),
        }
    }

    pub fn get_or_create_subtopic(&self, level: &str) -> &Topic {
        let mut subtopics = self.subtopics.write().unwrap();
        subtopics
            .entry(level.to_string())
            .or_insert_with(Topic::new)
    }

    pub fn add_subscriber(&self, client_id: Vec<u8>, data: SubscriptionData) {
        let mut subscribers = self.subscribers.write().unwrap();
        subscribers.insert(client_id, data);
    }

    pub fn remove_subscriber(&self, client_id: Vec<u8>) {
        let mut subscribers = self.subscribers.write().unwrap();
        subscribers.remove(&client_id);
    }

    pub fn add_retained_message(&self, message: Message) {
        let mut retained_messages = self.retained_messages.write().unwrap();
        retained_messages.push(message);
    }
}

pub struct TopicHandler {
    root: Topic,
    client_accions_receiver_channel: mpsc::Receiver<TopicHandlerTask>,
    clients: HashMap<Vec<u8>, Client>,
    active_connections: HashSet<i32>,
}

impl TopicHandler {
    pub fn new(receiver_channel: mpsc::Receiver<TopicHandlerTask>) -> Self {
        TopicHandler {
            root: Topic::new(),
            client_accions_receiver_channel: receiver_channel,
            clients: HashMap::new(),
            active_connections: HashSet::new(),
        }
    }

    pub fn run(self) {
        loop {
            match self.client_accions_receiver_channel.recv() {
                Ok(task) => match task {
                    TopicHandlerTask::SubscribeClient(subscribe, client_id) => {
                        self.subscribe(subscribe, client_id);
                    }
                    TopicHandlerTask::UnsubscribeClient(unsubscribe, client_id) => {
                        self.unsubscribe(unsubscribe);
                    }
                    TopicHandlerTask::Publish(publish, client_id) => {
                        self.publish(publish, client_id);
                    }
                    TopicHandlerTask::RegisterPubAck(puback) => {
                        self.register_puback(puback);
                    }
                    TopicHandlerTask::ClientConnected(client) => {
                        self.handle_client_connected(client);
                    }
                    TopicHandlerTask::ClientDisconnected(client_id) => {
                        self.handle_client_disconnected(client_id);
                    }
                },
                Err(_) => {
                    std::thread::sleep(Duration::from_secs(1));
                }
            }
        }
    }

    // Subscribe a client_id into a set of topics given a Subscribe packet
    // todo: replace the unwraps
    pub fn subscribe(&self, packet: Subscribe, client_id: Vec<u8>) {
        for (topic_filter, qos) in packet.topics {
            let mut topic = &self.root;
            let data = SubscriptionData { qos };
            for level in topic_filter.levels {
                match level {
                    TopicLevel::Literal(level_bytes) => {
                        let level = String::from_utf8(level_bytes).unwrap();
                        topic = topic.get_or_create_subtopic(&level);
                    }
                    TopicLevel::SingleLevelWildcard => {
                        let subtopics = topic.subtopics.read().unwrap();
                        for subtopic in subtopics.values() {
                            subtopic.add_subscriber(client_id.clone(), data);
                        }
                        topic = &self.root;
                    }
                    TopicLevel::MultiLevelWildcard => {
                        topic.add_subscriber(client_id.clone(), data);
                        subscribe_to_all_subtopics(topic, client_id.clone(), data);
                    }
                }
            }
            topic.add_subscriber(client_id.clone(), data);
        }
    }

    // Unsubscribe a client_id from a set of topics given an Unsubscribe packet
    pub fn unsubscribe(&self, unsubscribe_packet: Unsubscribe) {
        todo!()
    }

    // Publish a message to a topic given a Publish packet
    pub fn publish(&self, publish_packet: Publish, client_id: Vec<u8>) {
        let topic_name = publish_packet.topic;
        let topic_levels = topic_name.levels();

        let mut current_topic = &self.root;
        for level in topic_levels {
            let level_str = String::from_utf8_lossy(level); 
            current_topic = current_topic.get_or_create_subtopic(&level_str);
        }

        if publish_packet.retain {
            let retained_message = Message {
                client_id: client_id.clone(),
                packet: publish_packet,
            };
            current_topic.add_retained_message(retained_message);
        }

        self.publish_to_subscribers(current_topic, &publish_packet, client_id);
        
    }

    pub fn publish_to_subscribers(&self, topic: &Topic, publish_packet: &Publish, client_id: Vec<u8>) {
        let subscribers = topic.subscribers.read().unwrap();
        for (subscriber_id, subscription_data) in subscribers.iter() {
            if subscription_data.qos >= publish_packet.qos {
                if let Some(client) = self.clients.get(subscriber_id) {
                    client.send_packet(publish_packet);
                } else {
                    println!("Client {} not found", String::from_utf8_lossy(subscriber_id));
                }
            } 

        }
    }

    pub fn register_puback(&self, puback_packet: Puback) {
        todo!()
    }

    pub fn handle_client_connected(&self, client: Client) {
        todo!()
    }

    pub fn handle_client_disconnected(&self, client_id: Vec<u8>) {
        todo!()
    }
}

pub fn subscribe_to_all_subtopics(topic: &Topic, client_id: Vec<u8>, data: SubscriptionData) {
    topic.add_subscriber(client_id.clone(), data);
    let subtopics = topic.subtopics.read().unwrap();
    for subtopic in subtopics.values() {
        subscribe_to_all_subtopics(subtopic, client_id.clone(), data);
    }
}



