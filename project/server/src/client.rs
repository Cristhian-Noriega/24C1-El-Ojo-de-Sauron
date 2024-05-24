#![allow(dead_code)]
#![allow(unused_variables)]

use std::net::TcpStream;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use sauron::model::components::topic_name::TopicName;

// represents the state of the client in the server
#[derive(Debug)]
pub struct Client {
    pub id: Vec<u8>,
    pub password: String,
    pub subscriptions: Vec<TopicName>,
    pub alive: AtomicBool,
    pub stream: Mutex<TcpStream>, // ARC MUTEX TCP STREAM
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
            stream: Mutex::new(stream),
        }
    }

    pub fn add_subscription(&mut self, topic: TopicName ) {
        let client_id = String::from_utf8(self.id.clone()).unwrap();
        println!("Client with client id {:?} subscribed to {:?}\n", client_id, topic.clone().to_string());
        self.subscriptions.push(topic);
    }
}
