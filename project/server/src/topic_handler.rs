use std::collections::HashSet;
use std::error::Error;
use std::sync::{mpsc, RwLock};
use std::time::Duration;
use std::{collections::HashMap, io::Write};

// Los topics segun MQTT tienen una jerarquia, por ejemplo:
// home/livingroom/temperature. En este caso, home es el topico padre de livingroom
// y livingroom es el topico padre de temperature.
// es decir, hay una estructura de arbol en los topics

use sauron::model::qos::QoS;
use sauron::model::{
    packet::Packet,
    packets::{connack::Connack, puback::Puback, publish::Publish, subscribe::Subscribe},
};

use crate::client::Client;

pub enum TopicHandlerTask {
    ConnectNewClient(Client),
    SubscribeClient(Subscribe),
    UnsubscribeClient(Unsubscribe),
    Publish(Publish),
    RegisterPubAck(Puback),
    DisconnectClient(String),
}

pub struct SubscriptionData {
    qos: QoS,
}

pub struct Message {
    pub client_id: String,
    pub packet: Publish,
}

type Suscriber = HashMap<String, SubscriptionData>;

pub struct Topic {
    //name: String, el nombre del topico puede que ya venga en el paquete
    // use of RwLock to handle multiple readers and one writer
    // in this case, the server will have multiple clients that will read the topic
    //rwlock -> multiple threads can read the data in parallel but an exclusive access is needed for writing
    subscribers: RwLock<Suscriber>,
    // retained messages are messages that are stored in the server and sent to new subscribers
    // when they subscribe to the topic
    retained_messages: RwLock<Vec<Message>>, //
    // to support things like home/livingroom/temperature topics
    subtopics: RwLock<HashMap<String, Topic>>,
}

impl Topic {
    pub fn new() -> Self {
        Topic {
            subscribers: RwLock::new(HashMap::new()),
            retained_messages: RwLock::new(Vec::new()),
            subtopics: RwLock::new(HashMap::new()),
        }
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
                    TopicHandlerTask::ConnectNewClient(client) => {
                        self.connect_new_client(client);
                    }
                    TopicHandlerTask::SubscribeClient(subscribe) => {
                        self.subscribe(subscribe);
                    }
                    TopicHandlerTask::UnsubscribeClient(unsubscribe) => {
                        self.unsubscribe(unsubscribe);
                    }
                    TopicHandlerTask::Publish(publish) => {
                        self.publish(publish);
                    }
                    TopicHandlerTask::RegisterPubAck(puback) => {
                        self.register_puback(puback);
                    }
                    TopicHandlerTask::DisconnectClient(client_id) => {
                        self.disconnect_client(client_id);
                    }
                },
                Err(_) => {
                    std::thread::sleep(Duration::from_secs(1));
                }
            }
        }
    }

    // Subscribe a client_id into a set of topics given a Subscribe packet
    pub fn subscribe(&self, packet: Subscribe) {
        let client_id = packet.client_id;
        let topics = packet.get_topics();
        for topic in topics {
            let data = SubscriptionData { qos: topic.qos };
            subscribe_to_topic(&self.root, topic.topic_name, client_id, data);
        }
        Ok(());
    }

    pub fn connect_new_client(self, client: Client) {
        let client_id = client.id;
        if self.active_connections.contains(client.id) {
            println!("Client already connected: {:?}", client.id);
            return;
        } else {
            self.clients.insert(client.id.to_string(), client);
            self.active_connections.insert(client.id);
            println!("New client connected: {:?}", client.id);

            client.stream.write_all(Packet::Connack(
                Connack::new(true, "CONNECT CODE????").to_bytes(),
            ));
        }
    }
    pub fn unsubscribe(&self, unsubscribe_packet: Unsubscribe) {
        let client_id = unsubscribe_packet.client_id;
        let topics = unsubscribe_packet.get_topics();
        for topic in topics {
            unsubscribe_to_topic(&self.root, topic.topic_name, client_id);
        }
        Ok(());
    }

    pub fn publish(&self, publish_packet: Publish) {
        let topic_name = publish_packet.topic_name;
        let message = Message {
            client_id: publish_packet.client_id,
            packet: publish_packet,
        };
        publish_to_topic(&self.root, topic_name, message);
    }

    pub fn register_puback(&self, puback_packet: Puback) {
        //TODO: implementar
    }

    pub fn disconnect_client(&self, client_id: String) {
        self.clients.remove(&client_id);
        self.active_connections.remove(&client_id);
    }
}

pub fn subscribe_to_topic(
    current_topic: &Topic,
    topics: &str,
    client_id: &str,
    data: SubscriptionData,
) -> Result<(), Error> {
    let (topic, rest) = match topics.split_once("/") {
        Some((topic_name, rest)) => {
            // uso de write para obtener el rwlock en modo escritura
            let subtopics = current_topic.subtopics.read()?;
            if subtopics.get(topic_name).is_none() {
                drop(subtopics); //libero el lock de escritura antes de obtenerlo en modo escritura
                let mut subtopics = current_topic.subtopics.write()?;
                subtopics.insert(topic_name.to_string(), Topic::new());

                (subtopics.get(topic_name)?, rest);
            } else {
                //el topic ya existe
                (subtopics.get(topic_name)?, rest);
            }
        }
        None => {
            //caso base
            //si no hay mas subtopics, agrego el client_id a la lista de subscribers
            let mut subscribers = current_topic.subscribers.write()?;
            subscribers.insert(client_id.to_string(), data);
            return Ok(());
        }
    };
    //llamada recursiva para seguir bajando en la jerarquia de topics
    subscribe_to_topic(topic, rest, client_id, data)
}
