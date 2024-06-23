#![allow(unused_variables)]

use std::fmt;
use std::io::Write;
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use mqtt::model::components::topic_filter::TopicFilter;
use mqtt::model::components::topic_name::TopicName;
use mqtt::model::packets::publish::Publish;

/// Represents the state of the client in the server
#[derive(Debug)]
pub struct Client {
    pub id: Vec<u8>,
    pub subscriptions: Vec<TopicFilter>,
    pub alive: AtomicBool,
    pub stream: Arc<Mutex<TcpStream>>, // ARC MUTEX TCP STREAM
}

impl Client {
    pub fn new(id: Vec<u8>, stream: TcpStream, clean_session: bool, keep_alive: u16) -> Client {
        Client {
            id,
            subscriptions: Vec::new(),
            alive: AtomicBool::new(true),
            stream: Arc::new(Mutex::new(stream)),
        }
    }

    /// Subscribes the client to a topic
    pub fn add_subscription(&mut self, topic: TopicFilter) {
        let client_id = String::from_utf8(self.id.clone()).unwrap();
        println!(
            "Client with client id {:?} subscribed to {:?}",
            client_id,
            topic.clone().to_string()
        );
        self.subscriptions.push(topic);
    }

    /// Unsubscribes the client from a topic
    pub fn remove_subscription(&mut self, topic: &TopicFilter) {
        let client_id = String::from_utf8(self.id.clone()).unwrap();
        println!(
            "Client with client id {:?} unsubscribed from {:?}",
            client_id,
            topic.clone().to_string()
        );
        self.subscriptions.retain(|t| t != topic);
    }

    /// Checks if the client is subscribed to a topic
    pub fn is_subscribed(&self, topic: &TopicName) -> bool {
        self.subscriptions
            .iter()
            .any(|t| t.match_topic_name(topic.clone()))
    }

    /// Sends a message to the client
    pub fn send_message(
        &self,
        publish_packet: Publish,
        logfile: &Arc<crate::logfile::Logger>,
        key: &[u8],
    ) {
        let message_str = std::str::from_utf8(publish_packet.message()).unwrap();
        let client_id_str = std::str::from_utf8(&self.id).unwrap();

        let mut stream = self.stream.lock().unwrap();
        match stream.write_all(publish_packet.to_bytes(key).as_slice()) {
            Ok(_) => {
                logfile.log_sent_message(message_str, client_id_str);
            }
            Err(e) => logfile.log_sending_message_error(message_str, client_id_str),
        }
    }

    /// Gets the id of the client
    pub fn id(&self) -> Vec<u8> {
        self.id.clone()
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
            "Client ID: {}\nSubscriptions: {}\nAlive: {}",
            id,
            subscriptions,
            self.alive.load(Ordering::Relaxed)
        )
    }
}
