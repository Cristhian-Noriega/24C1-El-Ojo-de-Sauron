#![allow(dead_code)]
#![allow(unused_variables)]

use std::net::TcpStream;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

// represents the state of the client in the server

pub struct Client {
    pub id: Vec<u8>,
    pub password: String,
    pub subscriptions: Vec<String>,
    pub alive: AtomicBool,
    pub stream: Mutex<TcpStream>,
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
}
