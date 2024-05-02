pub struct SubscriptionData {
    client: Client,
    qos: QoSLevel,
}

pub struct Topic {
    name: String,
    subscribers: HashMap<String, SubscriptionData>,
    // retained messages are messages that are stored in the server and sent to new subscribers
    // when they subscribe to the topic 
    retained_messages: Vec<Message>,
}

impl Topic {
    pub fn new(name: String) -> Self {
        Topic {
            name,
            subscribers: Vec::new(),
            retained_messages: Vec::new(),
        }
    }
}

pub struct TopicHandler {
    topics: HashMap<String, Topic>
}

impl TopicHandler {
    pub fn new() -> Self {
        TopicHandler {
            topic: Topic {
                name: String::from(""),
                subscribers: Vec::new(),
                retained_messages: Vec::new(),
            }
        }
    }

    pub fn subscribe(&mut self, topic_name: &str, client: &Client, qos: QoSLevel) -> Result<(), Error> {
        let topic = self.topics.entry(topic_name.clone()).or_insert({
            let new_topic = Topic::new(topic_name.clone())?;
            new_topic
        });
        let subscription_data = SubscriptionData { client: client.clone(), qos };
        topic.subscribers.insert(client.id.clone(), subscription_data);
        Ok(())
    }
}
