use std::error::Error;
use std::sync::{mpsc, Arc, RwLock};
use std::time::Duration;
use std::collections::{HashMap, HashSet};

use sauron::model::encoded_string::EncodedString;
use sauron::model::qos::QoS;
use sauron::model::packets::{puback::Puback, publish::Publish, subscribe::Subscribe, unsubscribe::Unsubscribe};
use sauron::model::topic_filter::TopicFilter;
use sauron::model::topic_level::TopicLevel;
use crate::client::Client;

pub enum TopicHandlerTask {
    //todo: cambiarlo a nombres con acknoledgement al connect y disconnect
    //ConnectNewClient(Client), no es responsabilidad del topic handler la conexion
    SubscribeClient(Subscribe, Vec<u8>),
    UnsubscribeClient(Unsubscribe, Vec<u8>),
    Publish(Publish, Vec<u8>),
    RegisterPubAck(Puback),
    //DisconnectClient(String), no es responsabilidad del topic handler la desconexion
}

pub struct SubscriptionData {
    qos: QoS,
}

pub struct Message {
    pub client_id: EncodedString,
    pub packet: Publish,
}

type Suscriber = HashMap<String, SubscriptionData>;
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
        subtopics.entry(level.to_string()).or_insert_with(Topic::new)
    }

    pub fn add_subscriber(&self, client_id: String, data: SubscriptionData) {
        let mut subscribers = self.subscribers.write().unwrap();
        subscribers.insert(client_id, data);
    }

    pub fn remove_subscriber(&self, client_id: String) {
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
    clients: HashMap<String, Client>,
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
                    // TopicHandlerTask::ConnectNewClient(client) => {
                    //     self.connect_new_client(client);
                    // }
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
                    // TopicHandlerTask::DisconnectClient(client_id) => {
                    //     self.disconnect_client(client_id);
                    // }
                },
                Err(_) => {
                    std::thread::sleep(Duration::from_secs(1));
                }
            }
        }
    }

    // Subscribe a client_id into a set of topics given a Subscribe packet
    pub fn subscribe(&self, packet: Subscribe, client_id: Vec<u8>) {
        //let topics = packet.topics.iter().collect();
        let client_id = String::from_utf8(client_id).unwrap();
        for (topic_filter, qos) in packet.topics {
            let mut topic = &self.root;
            let data = SubscriptionData { qos };
            for level in &topic_filter.levels {
                match level {
                    TopicLevel::Literal(level_bytes) => {
                        let level = String::from_utf8_lossy(level_bytes);
                        topic = topic.get_or_create_subtopic(&level);
                    }
                    TopicLevel::SingleLevelWildcard  => {
                        todo!()
                    }
                    TopicLevel::MultiLevelWildcard => {
                        todo!()
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
        let topic_name = publish_packet.topic_name.to_string();
        let topic_levels: Vec<&str> = topic_name.split('/').collect();
        let client_id = String::from_utf8(client_id).unwrap();

        let mut current_topic = &self.root;
        for level in topic_levels {
            current_topic = current_topic.get_or_create_subtopic(level);
        }
        // if publish_packet.retain {
        //     let message = Message {
        //         client_id,
        //         packet: publish_packet,
        //     };
        //     current_topic.add_retained_message(message);
        // }
        let mut subscribers = current_topic.subscribers.write().unwrap();
        // todo: here i should send the message to the subscribers
    }

    // Register a Puback packet
    pub fn register_puback(&self, puback_packet: Puback) {
        todo!()
    }


}



//     // Subscribe a client_id into a set of topics given a Subscribe packet
//     pub fn subscribe(&self, packet: Subscribe) {
//         let client_id = packet.client_id;
//         let topics = packet.get_topics();
//         for topic in topics {
//             let data = SubscriptionData { qos: topic.qos };
//             subscribe_to_topic(&self.root, topic.topic_name, client_id, data);
//         }
//         Ok(());
//     }

//     pub fn connect_new_client(self, client: Client) {
//         let client_id = client.id;
//         if self.active_connections.contains(client.id) {
//             println!("Client already connected: {:?}", client.id);
//             return;
//         } else {
//             self.clients.insert(client.id.to_string(), client);
//             self.active_connections.insert(client.id);
//             println!("New client connected: {:?}", client.id);

//             client.stream.write_all(Packet::Connack(
//                 Connack::new(true, "CONNECT CODE????").to_bytes(),
//             ));
//         }
//     }
//     pub fn unsubscribe(&self, unsubscribe_packet: Unsubscribe) {
//         let client_id = unsubscribe_packet.client_id;
//         let topics = unsubscribe_packet.get_topics();
//         for topic in topics {
//             unsubscribe_to_topic(&self.root, topic.topic_name, client_id);
//         }
//         Ok(());
//     }

//     pub fn publish(&self, publish_packet: Publish) {
//         let topic_name = publish_packet.topic_name;
//         let message = Message {
//             client_id: publish_packet.client_id,
//             packet: publish_packet,
//         };
//         publish_to_topic(&self.root, topic_name, message);
//     }

//     pub fn register_puback(&self, puback_packet: Puback) {
//         //TODO: implementar
//     }

//     pub fn disconnect_client(&self, client_id: String) {
//         self.clients.remove(&client_id);
//         self.active_connections.remove(&client_id);
//     }
// }

// pub fn subscribe_to_topic(
//     current_topic: &Topic,
//     topics: &str,
//     client_id: &str,
//     data: SubscriptionData,
// )  {
//     todo!()
//     // let (topic, rest) = match topics.split_once("/") {
//     //     Some((topic_name, rest)) => {
//     //         // uso de write para obtener el rwlock en modo escritura
//     //         let subtopics = current_topic.subtopics.read()?;
//     //         if subtopics.get(topic_name).is_none() {
//     //             drop(subtopics); //libero el lock de escritura antes de obtenerlo en modo escritura
//     //             let mut subtopics = current_topic.subtopics.write()?;
//     //             subtopics.insert(topic_name.to_string(), Topic::new());

//     //             (subtopics.get(topic_name)?, rest);
//     //         } else {
//     //             //el topic ya existe
//     //             (subtopics.get(topic_name)?, rest);
//     //         }
//     //     }
//     //     None => {
//     //         //caso base
//     //         //si no hay mas subtopics, agrego el client_id a la lista de subscribers
//     //         let mut subscribers = current_topic.subscribers.write()?;
//     //         subscribers.insert(client_id.to_string(), data);
//     //         return Ok(());
//     //     }
//     // };
//     // //llamada recursiva para seguir bajando en la jerarquia de topics
//     // subscribe_to_topic(topic, rest, client_id, data)
// }