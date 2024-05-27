#![allow(dead_code)]
#![allow(unused_variables)]

use std::fmt;
use std::io::Write;
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use mqtt::model::components::topic_name::TopicName;

use crate::task_handler::Message;

// represents the state of the client in the server
#[derive(Debug)]
pub struct Client {
    pub id: Vec<u8>,
    pub password: String,
    pub subscriptions: Vec<TopicName>,
    pub alive: AtomicBool,
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

    pub fn add_subscription(&mut self, topic: TopicName) {
        let client_id = String::from_utf8(self.id.clone()).unwrap();
        println!(
            "Client with client id {:?} subscribed to {:?}",
            client_id,
            topic.clone().to_string()
        );
        self.subscriptions.push(topic);
    }

    pub fn remove_subscription(&mut self, topic: &TopicName) {
        let client_id = String::from_utf8(self.id.clone()).unwrap();
        println!(
            "Client with client id {:?} unsubscribed from {:?}",
            client_id,
            topic.clone().to_string()
        );
        self.subscriptions.retain(|t| t != topic);
    }

    pub fn send_message(&self, message: Message) {
        let mut stream = self.stream.lock().unwrap();
        match stream.write_all(message.packet().to_bytes().as_slice()) {
            Ok(_) => println!("Message sent to client"),
            Err(e) => println!("Failed to send message: {}", e),
        }
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
