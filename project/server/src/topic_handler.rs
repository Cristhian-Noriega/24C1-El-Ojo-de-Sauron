use std::collections::HashMap;
use std::sync::RwLock;

// Los topics segun MQTT tienen una jerarquia, por ejemplo:
// home/livingroom/temperature. En este caso, home es el topico padre de livingroom
// y livingroom es el topico padre de temperature. 
// es decir, hay una estructura de arbol en los topics
// 




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
    pub fn new() -> Self {
        Topic {
            subscribers: RwLock::new(HashMap::new()),
            retained_messages: RwLock::new(Vec::new()),
            subtopics: RwLock::new(HashMap::new()),
        }
    }
}

pub struct TopicHandler {
    root: Topic
}

impl TopicHandler {
    pub fn new() -> Self {
        TopicHandler {
            root: Topic::new()
        }
    }

    // Subscribe a client_id into a set of topics given a Subscribe packet
    pub fn subscribe(&self, client_id: &str, packet: &Subscribe) -> Result<Option<>, Error> {
        let topics = packet.get_topics();
        for topic in topics {
            let data = SubscriptionData {
                qos: topic.qos
            };
            subscribe_to_topic(&self.root, topic.topic_name, client_id, data);
        }
        Ok(());
    } 
}

pub fn subscribe_to_topic(current_topic: &Topic, topics: &str, client_id: &str, data: SubscriptionData) -> Result<(), Error>{
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
   


