#![allow(dead_code)]

use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use std::net::TcpStream;

// represents the state of the client in the server 

pub struct Client { 
    pub id: String,
    pub password: String,
    pub subscriptions: Vec<String>,
    pub alive: AtomicBool,
    pub stream: Mutex<TcpStream>,
}

impl Client {
    pub fn new(
        id: String, 
        password: String, 
        stream: TcpStream,
        clean_session: bool,
        keep_alive: u16, 
        // will: Option<(QoS, String, String)>, 
        // user: Option<(String, Option<String>)>
    ) -> Client {
        Client {
            id,
            password,
            subscriptions: Vec::new(),
            alive: AtomicBool::new(true),
            stream: Mutex::new(stream),
        }
    }
}
