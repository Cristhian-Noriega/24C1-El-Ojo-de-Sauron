#![allow(dead_code)]

use std::io::Write;
use std::sync::atomic::AtomicBool;
use std::sync::{mpsc, Mutex};
use std::net::TcpStream;

use crate::server::server::Packet;
// use crate::connect;
// use sauron::connect as sauron_connect;
// use sauron::subscribe as sauron_subscribe;
// use crate::model::package_components::fixed_header_components::qos::QoS;

//represents the state of the client in the server 

pub struct Client { 
    id: String,
    password: String,
    subscriptions: Vec<String>,
    //log: Vec<package>,
    // alive is an atomic bool to avoid race conditions
    alive: AtomicBool,
    // Channel between server thread and client thread
    channel: mpsc::Sender<ClientTask>,

    // the stream represents the communication channel between the client and the server
    // throught the client will received and send data
    // it is wrapped in a mutex for thread safety
    stream: Mutex<TcpStream>,
    //connect: Packet,
}

pub enum ClientTask{
    SendConnack,
    SendPublish,
    SendPuback,
    SendSubscribe,
    SendUnsubscribe,
    SendPingreq,
    SendDisconnect,
}


impl Client {
    pub fn new(
        id: String, 
        password: String, 
        stream: TcpStream, 
        channel: mpsc::Sender<ClientTask>,
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
            channel,
        }
    }


    pub fn send_task(&self, task: ClientTask) {
        let channel = self.channel.as_ref().unwrap();
        channel.send(task).unwrap();
    }

    //ACA ESTÁ LA MAGIA DE LOS CLIENT THREADS Y LAS OPERACIONES QUE REALIZAN
    // manda por su stream el package suback
    pub fn stream_packet(&self, packet: Packet) -> std::io::Result<()> {
        let packet_bytes = packet.into_bytes();
        let mut stream = self.stream.lock().unwrap();
        stream.write_all(&packet_bytes)
    }
    

// ESTO NO VA EN LA CARPETA DE CLIENTE???
// Connects the client to the server by sending a connect package to the server
    // pub fn connect(&self) -> std::io::Result<()> {
    //     let connect_bytes = self.connect.into_bytes();
    //     let mut stream = self.stream.lock().unwrap();
    //     stream.write_all(&connect_bytes)
    // }

    // pub fn suscribe(&self, topic: String) {
    //     let package = sauron_subscribe(self.id.clone(), vec![(topic_name.to_string(), qos)]);

    //     let mut topic_handler = self.topic_handler.lock().unwrap();
    //     topic_handler.subscribe(topic_name, self, qos)?;

    //     self.send(package)
    // }

    // pub fn publish(&self, topic: String, message: String) -> std::io::Result<()> {
    //     let publish = sauron_publish(topic, message);
    //     let publish_bytes = publish.into_bytes();
    //     let mut stream = self.stream.lock().unwrap();
    //     stream.write_all(&publish_bytes)
    // }
}
