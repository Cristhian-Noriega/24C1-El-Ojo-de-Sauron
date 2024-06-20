#![allow(dead_code)]
#![allow(unused_variables)]

use std::fmt;
use std::io::Write;
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use mqtt::model::components::topic_filter::TopicFilter;
use mqtt::model::components::topic_name::TopicName;

use crate::task_handler::Message;

// represents the state of the client in the server
#[derive(Debug)]
pub struct Client {
    id: Vec<u8>,
    password: String,
    subscriptions: Vec<TopicFilter>,
    alive: AtomicBool,
    pub stream: Arc<Mutex<TcpStream>>, // ARC MUTEX TCP STREAM
}

impl Client {
    pub fn new(
        id: Vec<u8>,
        password: String,
        stream: TcpStream,
        clean_session: bool,
        keep_alive: u16,
    ) -> Client {
        Client {
            id,
            password,
            subscriptions: Vec::new(),
            alive: AtomicBool::new(true),
            stream: Arc::new(Mutex::new(stream)),
        }
    }

    pub fn add_subscription(&mut self, topic: TopicFilter) {
        let client_id = String::from_utf8(self.id.clone()).unwrap();
        println!(
            "Client with client id {:?} subscribed to {:?}",
            client_id,
            topic.clone().to_string()
        );
        self.subscriptions.push(topic);
    }

    pub fn remove_subscription(&mut self, topic: &TopicFilter) {
        let client_id = String::from_utf8(self.id.clone()).unwrap();
        println!(
            "Client with client id {:?} unsubscribed from {:?}",
            client_id,
            topic.clone().to_string()
        );
        self.subscriptions.retain(|t| t != topic);
    }

    pub fn is_subscribed(&self, topic: &TopicName) -> bool {
        self.subscriptions
            .iter()
            .any(|t| t.match_topic_name(topic.clone()))
    }

    pub fn send_message(&self, message: Message, logfile: &Arc<crate::logfile::Logger>) {
        let message_str = std::str::from_utf8(message.packet().message()).unwrap();
        let client_id_str = std::str::from_utf8(&self.id).unwrap();

        let mut stream = self.stream.lock().unwrap();
        match stream.write_all(message.packet().to_bytes().as_slice()) {
            Ok(_) => {
                logfile.log_sent_message(message_str, client_id_str);
            }
            Err(e) => logfile.log_sending_message_error(message_str, client_id_str),
        }
    }

    pub fn id(&self) -> Vec<u8> {
        self.id.clone()
    }

    pub fn stream(&self) -> Arc<Mutex<TcpStream>> {
        self.stream.clone()
    }
}

impl fmt::Display for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id = String::from_utf8_lossy(&self.id);
        let subscriptions = self
            .subscriptions
            .iter()
            .map(|topic| topic.to_string())
            .collect::<Vec<String>>()
            .join(", ");

        write!(
            f,
            "Client ID: {}\nPassword: {}\nSubscriptions: {}\nAlive: {}",
            id,
            self.password,
            subscriptions,
            self.alive.load(Ordering::Relaxed)
        )
    }
}

#[cfg(test)]
mod tests {
    use std::{net::TcpListener, vec};

    use mqtt::model::components::topic_level::TopicLevel;

    use super::*;
    
    fn setup_stream() -> TcpStream {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();  
        let port = listener.local_addr().unwrap().port();  
        let stream = TcpStream::connect(format!("127.0.0.1:{}", port)).unwrap();
        listener.accept().unwrap().0
    }

    fn setup_client() -> Client {
        let stream = setup_stream();
        Client::new(
            vec![1, 2, 3],
            "password".to_string(),
            stream,
            true,
            60,
        )
    }

    fn setup_topic_filter() -> Vec<TopicFilter> {
        let mut topic_filters = vec![];
        let topic_level1 = TopicLevel::Literal(b"topic".to_vec());
        let topic_level2 = TopicLevel::Literal(b"level".to_vec());
        let topic_level3 = TopicLevel::Literal(b"home".to_vec());
        let topic_level4 = TopicLevel::Literal(b"livingroom".to_vec());

        let topic_filter1 = TopicFilter::new(vec![topic_level1, topic_level2], false);
        let topic_filter2 = TopicFilter::new(vec![topic_level3, topic_level4], false);
       
        topic_filters.push(topic_filter1);
        topic_filters.push(topic_filter2); 
        
        topic_filters
    }


    #[test]
    fn test_new_client() {
        let client = setup_client();
        assert_eq!(client.id, vec![1, 2, 3]);
        assert_eq!(client.password, "password");
        assert!(client.subscriptions.is_empty());
        assert_eq!(client.alive.load(Ordering::Relaxed), true);
    }

    #[test]
    fn test_add_subscription() {
        let mut client = setup_client();
        let topic = setup_topic_filter();
        client.add_subscription(topic[0].clone());
        assert_eq!(client.subscriptions.len(), 1);
        assert_eq!(client.subscriptions[0], topic[0]);
    }

    #[test]
    fn test_remove_subscription() {
        let mut client = setup_client();
        let topic = setup_topic_filter();
        client.add_subscription(topic[0].clone()); 
        client.remove_subscription(&topic[0]);
        assert!(client.subscriptions.is_empty());
    }

    #[test]
    fn test_is_subscribed() {
        let mut client = setup_client();
        let topic_filter = setup_topic_filter();
        client.add_subscription(topic_filter[0].clone());
        let topic_name = TopicName::new(vec![b"topic".to_vec()], false);
        assert!(!client.is_subscribed(&topic_name));
        let topic_name = TopicName::new(vec![b"topic".to_vec(), b"level".to_vec()], false);
        assert!(client.is_subscribed(&topic_name));
    }

    #[test]
    fn test_adding_multiple_subscriptions() {
        let mut client = setup_client();
        let topic_filter = setup_topic_filter();
        client.add_subscription(topic_filter[0].clone());
        client.add_subscription(topic_filter[1].clone());
        assert_eq!(client.subscriptions.len(), 2);
        assert_eq!(client.subscriptions[0], topic_filter[0]);
        assert_eq!(client.subscriptions[1], topic_filter[1]);
    }
}
