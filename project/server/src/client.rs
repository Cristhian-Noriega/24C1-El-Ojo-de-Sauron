#![allow(dead_code)]

use std::net::TcpStream;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use std::collections::VecDeque;



use sauron::model::packets::publish::Publish;

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
