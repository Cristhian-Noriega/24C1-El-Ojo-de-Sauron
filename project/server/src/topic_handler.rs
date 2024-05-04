use std::collections::HashMap;
use std::sync::RwLock;



pub struct SubscriptionData {
    qos: QoSLevel,
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
    pub fn new(name: String) -> Self {
        Topic {
            subscribers: RwLock::new(HashMap::new()),
            retained_messages: RwLock::new(Vec::new()),
            subtopics: RwLock::new(HashMap::new()),
        }
    }
}

pub struct TopicHandler {
    topics: HashMap<String, Topic>
}

impl TopicHandler {
    pub fn new() -> Self {
        TopicHandler {
            topics: HashMap::new()
        }
    }

    // pub fn subscribe(&mut self, topic_name: &str, client: &Client, qos: QoSLevel) -> Result<(), Error> {
    //     let topic = self.topics.entry(topic_name.clone()).or_insert({
    //         let new_topic = Topic::new(topic_name.clone())?;
    //         new_topic
    //     });
    //     let subscription_data = SubscriptionData { client: client.clone(), qos };
    //     topic.subscribers.insert(client.id.clone(), subscription_data);
    //     Ok(())
    // }

    pub fn suscribe(&self, packet: &Suscribe, client_id: &str) -> Result<Option<>, Error> {
        let topics = packet.topics();
        let topics = topics.split("/");

        for topic in topics {
            let topic = self.topics.entry(topic).or_insert({
                let new_topic = Topic::new(topic)?;
                new_topic
            });
            let subscription_data = SubscriptionData { client: client.clone(), qos };
            topic.subscribers.insert(client.id.clone(), subscription_data);
        }
        Ok(())
    }
}
