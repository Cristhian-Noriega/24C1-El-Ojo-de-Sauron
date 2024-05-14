#![allow(dead_code)]

use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use std::net::TcpStream;

// represents the state of the client in the server 

pub struct Client { 
    id: String,
    password: String,
    subscriptions: Vec<String>,
    alive: AtomicBool,

    // the stream represents the communication channel between the client and the server
    // throught the client will received and send data
    // it is wrapped in a mutex for thread safety
    stream: Mutex<TcpStream>,
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
        //let connect = sauron_connect(id.clone(), clean_session, keep_alive, will, user);
        Client {
            id,
            password,
            subscriptions: Vec::new(),
            //log: Vec::new(),
            alive: true,
            stream: Mutex::new(stream),
        }
    }
}
