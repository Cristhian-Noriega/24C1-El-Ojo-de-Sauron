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
    components::{qos::QoS, topic_level::TopicLevel, topic_name::TopicName},
    packets::{publish::Publish, subscribe::Subscribe, unsubscribe::Unsubscribe},
};

pub enum Task {
    SubscribeClient(Subscribe, Vec<u8>),
    UnsubscribeClient(Unsubscribe, Vec<u8>),
    Publish(Publish, Vec<u8>),
    ClientConnected(Client),
    ClientDisconnected(Vec<u8>),
}
#[derive(Clone)]
pub struct SubscriptionData {
    qos: QoS,
}

pub struct Message {
    pub client_id: Vec<u8>,
    pub packet: Publish,
}

type Subscribers = HashMap<Vec<u8>, SubscriptionData>; // key : client_id , value: SubscriptionData
type Subtopic = HashMap<Vec<u8>, Topic>; // key: level, value: Topic
type Subscriptions = HashMap<TopicName, SubscriptionData>; // key: topic_name, value: SubscriptionData

pub struct Topic {
    subscribers: RwLock<Subscribers>,
    retained_messages: RwLock<Vec<Message>>,
    subtopics: RwLock<Subtopic>,
    subscriptions: RwLock<Subscriptions>,
}

impl Topic {
    pub fn new() -> Self {
        Topic {
            subscribers: RwLock::new(HashMap::new()),
            retained_messages: RwLock::new(Vec::new()),
            subtopics: RwLock::new(HashMap::new()),
            subscriptions: RwLock::new(HashMap::new()),
        }
    }

    // todo: replace the unwraps
    pub fn subscribe(
        &self,
        topic: &Topic,
        mut levels: Vec<TopicLevel>,
        client_id: Vec<u8>,
        data: SubscriptionData,
    ) {
        if levels.is_empty() {
            self.add_subscriber(client_id.clone(), data.clone());
            return;
        }
        let level = levels.remove(0);
        match level {
            TopicLevel::Literal(level_bytes) => {
                let subtopics = self.subtopics.read().unwrap();
                if let Some(subtopic) = subtopics.get(&level_bytes) {
                    self.subscribe(subtopic, levels.clone(), client_id, data);
                } else {
                    let mut subtopics = self.subtopics.write().unwrap();
                    subtopics.insert(level_bytes.clone(), Topic::new());
                    let subtopic = subtopics.get(&level_bytes).unwrap();
                    self.subscribe(subtopic, levels.clone(), client_id, data);
                }
            }
            TopicLevel::SingleLevelWildcard => {
                let subtopics = self.subtopics.read().unwrap();
                for subtopic in subtopics.values() {
                    self.subscribe(subtopic, levels.clone(), client_id.clone(), data.clone());
                }
            }
            TopicLevel::MultiLevelWildcard => {
                topic.add_subscriber(client_id.clone(), data.clone());
                subscribe_to_all_subtopics(topic, client_id, &data)
            }
        }
    }

    pub fn publish(
        &self,
        topic_name: TopicName,
        message: Message,
        clients: &HashMap<Vec<u8>, Client>,
        active_connections: &HashSet<Vec<u8>>,
    ) {
        let subscribers = self.get_all_matching_subscriptions(topic_name);
        for subscriber in subscribers {
            for (client_id, data) in subscriber {
                // if !active_connections.contains(&client_id){
                //     clients.get(&client_id).unwrap().unreceived_messages.(message.packet.clone());
                //     continue;
                // }
                let client = clients.get(&client_id).unwrap();

                let publish_packet = message.packet.clone();
                let _ = match client
                    .stream
                    .lock()
                    .unwrap()
                    .write(publish_packet.to_bytes().as_slice())
                {
                    Ok(_) => {}
                    Err(_) => {}
                };
            }
        }
    }

    pub fn get_all_matching_subscriptions(&self, topic_name: TopicName) -> Vec<Subscribers> {
        let mut subscribers = Vec::new();
        self.collect_matching_subscriptions(&mut subscribers, topic_name.levels);
        subscribers
    }

    pub fn collect_matching_subscriptions(
        &self,
        subscribers: &mut Vec<Subscribers>,
        levels: Vec<Vec<u8>>,
    ) {
        if levels.is_empty() {
            subscribers.push(self.subscribers.read().unwrap().clone());
            return;
        }
        let current_level = &levels[0];
        let remaining_levels = levels[1..].to_vec();

        let subtopics = self.subtopics.read().unwrap();
        if let Some(subtopic) = subtopics.get(current_level) {
            subtopic.collect_matching_subscriptions(subscribers, remaining_levels);
        }
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

pub struct TaskHandler {
    root: Topic,
    //client_actions_sender_channel: mpsc::Sender<Message>,
    client_actions_receiver_channel: mpsc::Receiver<Task>,
    clients: HashMap<Vec<u8>, Client>,
    active_connections: HashSet<Vec<u8>>,
    retained_messages: HashMap<Vec<u8>, Publish>,
}

impl TaskHandler {
    pub fn new(receiver_channel: mpsc::Receiver<Task>) -> Self {
        TaskHandler {
            root: Topic::new(),
            //client_actions_sender_channel: sender_channel,
            client_actions_receiver_channel: receiver_channel,
            clients: HashMap::new(),
            active_connections: HashSet::new(),
            retained_messages: HashMap::new(),
        }
    }

    pub fn run(self) {
        loop {
            match self.client_actions_receiver_channel.recv() {
                Ok(task) => match task {
                    Task::SubscribeClient(subscribe, client_id) => {
                        println!(
                            "Topic Handler received task: subscribe Client: {:?}",
                            client_id
                        );
                        self.subscribe(subscribe, client_id);
                    }
                    Task::UnsubscribeClient(unsubscribe, client_id) => {
                        println!(
                            "Topic Handler received task: unsubscribe Client: {:?}",
                            client_id
                        );
                        self.unsubscribe(unsubscribe);
                    }
                    Task::Publish(publish, client_id) => {
                        println!(
                            "Topic Handler received task: Publish message: {:?}",
                            publish
                        );
                        self.publish(&publish, client_id);
                    }
                    // TopicHandlerTask::RegisterPubAck(puback) => {
                    //     self.register_puback(puback);
                    // }
                    Task::ClientConnected(client) => {
                        self.handle_client_connected(client);
                    }
                    Task::ClientDisconnected(client_id) => {
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

    pub fn subscribe(&self, subscribe_packet: Subscribe, client_id: Vec<u8>) {
        let topics = subscribe_packet.topics;
        for (topic_filter, qos) in topics {
            let data = SubscriptionData { qos };
            self.root.subscribe(
                &self.root,
                topic_filter.levels,
                client_id.clone(),
                data.clone(),
            )
        }
    }

    // Unsubscribe a client_id from a set of topics given an Unsubscribe packet
    pub fn unsubscribe(&self, unsubscribe_packet: Unsubscribe) {
        todo!()
    }

    /*publish uses a publish method of the topic struct and also sends to the clients subscribed to the topic the message*/
    pub fn publish(&self, publish_packet: &Publish, client_id: Vec<u8>) {
        let topic_name = publish_packet.topic.clone();
        let message = Message {
            client_id: client_id.clone(),
            packet: publish_packet.clone(),
        };
        self.root
            .publish(topic_name, message, &self.clients, &self.active_connections);
    }

    pub fn handle_client_connected(&self, client: Client) {
        todo!()
    }

    pub fn handle_client_disconnected(&self, client_id: Vec<u8>) {
        todo!()
    }
}

pub fn subscribe_to_all_subtopics(topic: &Topic, client_id: Vec<u8>, data: &SubscriptionData) {
    topic.add_subscriber(client_id.clone(), data.clone());
    let subtopics = topic.subtopics.read().unwrap();
    for subtopic in subtopics.values() {
        subscribe_to_all_subtopics(subtopic, client_id.clone(), data);
    }
}
