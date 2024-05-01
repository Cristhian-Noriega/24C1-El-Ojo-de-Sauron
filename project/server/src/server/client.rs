use std::io::Write;
use std::sync::Mutex;
use std::net::TcpStream;
use crate::connect;
use crate::model::package::Package;
use crate::model::package_components::fixed_header_components::qos::QoS;


pub struct Client {
    id: String,
    password: String,
    subscriptions: Vec<String>,
    log: Vec<package>,
    alive: bool,
    // the stream represents the communication channel between the client and the server
    // throught the client will received and send data
    // it is wrapped in a mutex for thread safety
    stream: Mutex<TcpStream>,
    connect: Package,
}

impl Client {
    pub fn new(
        id: String, 
        password: String, 
        stream: TcpStream, 
        clean_session: bool, 
        keep_alive: u16, 
        will: Option<(QoS, String, String)>, 
        user: Option<(String, Option<String>)>
    ) -> Client {
        let connect = connect(id.clone(), clean_session, keep_alive, will, user);
        Client {
            id,
            password,
            subscriptions: Vec::new(),
            log: Vec::new(),
            alive: true,
            stream: Mutex::new(stream),
            connect,
        }
    }
// Connects the client to the server by sending a connect package to the server
    pub fn connect(&self) -> std::io::Result<()> {
        let connect_bytes = self.connect.into_bytes();
        let mut stream = self.stream.lock().unwrap();
        stream.write_all(&connect_bytes)
    }
}