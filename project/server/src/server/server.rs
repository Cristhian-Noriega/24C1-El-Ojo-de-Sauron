#![allow(dead_code)]
use std::{
    collections::{HashMap, HashSet}, io::BufReader, net::{TcpListener, TcpStream}, sync::{mpsc, Arc, Mutex}, thread
};
use crate::{client::Client, config::Config, topic_handler::{self, TopicHandler, TopicHandlerTask}};

use sauron::model::{packet::Packet, packets::{connect::Connect, disconnect::Disconnect, pingreq::Pingreq, pingresp::Pingresp, puback::Puback, publish::Publish, subscribe::Subscribe}};

pub struct Server {
    clients: HashMap<String, Client>,
    active_connections: HashSet<i32>,
    topic_handler: TopicHandler,
    config: Config,
    client_actions_sender_channel: mpsc::Sender<TopicHandlerTask>,
}

impl Server {
    pub fn new() -> Self {
        let (client_actions_sender_channel, client_accions_receiver_channel) = mpsc::channel();
        Server {
            clients: Vec::new(),
            topic_handler: TopicHandler::new(client_accions_receiver_channel),
            active_connections: HashSet::new(),
            config: Config::new("/."),
            client_actions_sender_channel,
        }
    }

    pub fn server_run(address: &str) -> std::io::Result<()> {
        let server = Server::new()?;
        let listener = TcpListener::bind(address)?;

        for stream_result in listener.incoming() {
            match stream_result {
                Ok(stream) => {
                    let address = stream.peer_addr()?.to_string();
                    println!("Nuevo paquete de la dirección: {:?}", address);
                    let mut reader = BufReader::new(stream);
                    let mut buffer = Vec::new();
            
                    reader.read_to_end(&mut buffer)?;
            
                    let mut cursor = std::io::Cursor::new(buffer);

                    let packet = Packet::from_bytes(&mut cursor)?;
            
                    println!("Packet recibidio desde la dirección: {:?}", address);
                    server.handle_packet_server(packet, stream);
                }
                Err(err) => {
                    println!("Error al recibir paquete: {:?}", err);
                }
            }
        }

        Ok(())
    }

    pub fn handle_packet_server(&self, packet: Packet, stream: TcpStream) {
        match packet {
            Packet::Connect(connect_packet) => self.handle_connect(connect_packet, stream),
            _ => println!("Unsupported packet type"),
        }
    }

    pub fn handle_connect(&self, connect_packet: Connect, stream: TcpStream) {
        let client_id = connect_packet.client_id().unwrap();

        if self.active_connections.contains(&client_id) {
            println!("Client already connected: {:?}", client_id);
            return;
        }
        else {
            let new_client = Client::new(client_id, 
                                                "PASSWORD".to_string(),
                                                Arc::new(Mutex::new(stream)),
                                                true, 
                                                0
                                                );
            self.clients.insert(client_id, new_client);
            self.active_connections.insert(client_id);
            self.create_new_client_thread(self.client_actions_sender_channel, stream);
            println!("New client connected: {:?}", client_id);
        }
    }

    pub fn create_new_client_thread(sender_to_topics_channel: std::sync::mpsc::Sender<TopicHandlerTask>, mut stream: TcpStream) {
        thread::spawn(move || {
            let address = stream.peer_addr().unwrap().to_string();
            loop {
                // Create a buffer to read incoming data
                let mut reader = BufReader::new(&mut stream);
                let mut buffer = Vec::new();

                match reader.read_to_end(&mut buffer) {
                    Ok(_) => {
                        let mut cursor = std::io::Cursor::new(buffer);
                        let packet: Packet = Packet::from_bytes(&mut cursor)?;
    
                        println!("Packet received from address: {:?}", address);
                        handle_packet(packet, stream, sender_to_topics_channel);
    
                        buffer.clear();
                    }
                    Err(err) => {
                        println!("Error reading from stream: {:?}", err);
                        break;
                    }
                }
            }
        });
    }
}

pub fn handle_packet(packet: Packet, client_id: Vec<u8>, sender_to_topics_channel: std::sync::mpsc::Sender<TopicHandlerTask>) {
    match packet {
        Packet::Publish(publish_packet)  => handle_publish(publish_packet, sender_to_topics_channel),
        Packet::Puback(puback_packet) => handle_puback(puback_packet, sender_to_topics_channel),
        Packet::Subscribe(subscribe_packet) => handle_subscribe(subscribe_packet, sender_to_topics_channel),
        Packet::Unsubscribe(unsubscribe_packet) => handle_unsubscribe(unsubscribe_packet, sender_to_topics_channel),
        Packet::Pingreq(pingrq_packet) => handle_pingreq(pingrq_packet),
        Packet::Disconnect(disconnect_packet) => handle_disconnect(disconnect_packet, sender_to_topics_channel),
        _ => {
            println!("Unsupported packet type");
            //kill_thread??(stream);
        },
    }
}
pub fn handle_publish(publish_packet: Publish, sender_to_topics_channel: std::sync::mpsc::Sender<TopicHandlerTask>) {
    let client_id = Publish::publish_packet::client_id().unwrap();
    let topic_name = publish_packet.topic_name().unwrap();
    sender_to_topics_channel.send(TopicHandlerTask::Publish(client_id, topic_name));
}

pub fn handle_puback(packet: Packet, sender_to_topics_channel: std::sync::mpsc::Sender<TopicHandlerTask>){
}


pub fn handle_subscribe(packet: Packet, sender_to_topics_channel: std::sync::mpsc::Sender<TopicHandlerTask>) {
}

pub fn handle_unsubscribe(packet: Packet, sender_to_topics_channel: std::sync::mpsc::Sender<TopicHandlerTask>) {

}

pub fn handle_disconnect(packet: Packet, sender_to_topics_channel: std::sync::mpsc::Sender<TopicHandlerTask>) {
    //kill_thread??(stream);
}

pub fn handle_pingreq(stream: TcpStream) {
    let pingresp_packet = Pingresp::new();
    let pingresp_bytes = pingresp_packet.into_bytes();
    stream.write_all(pingresp_bytes);
}

pub fn kill_thread(stream: TcpStream) {
    // LOGICA DE MATAR EL THREAD ACTUAL
}
