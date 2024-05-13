#![allow(dead_code)]
use std::{
    collections::{HashMap, HashSet, VecDeque}, io::BufReader, net::{Shutdown, TcpListener, TcpStream}, sync::mpsc::{self, TryRecvError}, thread
};
use crate::{client::{Client, ClientTask}, config::Config, topic_handler::{self, TopicHandler, TopicHandlerTask}};

use sauron::model::{packet::Packet, packets::{connect::Connect, disconnect::Disconnect, pingreq::Pingreq, puback::Puback, publish::Publish, subscribe::Subscribe}};

pub struct Server {
    clients: HashMap<String, Client>,
    active_connections: HashSet<i32>,
    topic_handler: TopicHandler,
    config: Config
}

impl Server {
    pub fn new() -> Self {
        Server {
            clients: Vec::new(),
            topic_handler: TopicHandler::new(),
            active_connections: HashSet::new(),
            config: Config::new("/."),
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
                    server.handle_packet_server(packet, address, stream);
                }
                Err(err) => {
                    println!("Error al recibir paquete: {:?}", err);
                }
            }
        }

        Ok(())
    }

    pub fn handle_packet_server(&self, packet: Packet, client_id: Vec<u8>, stream: TcpStream) {
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
            let (server_to_client_sender_channel, server_to_client_receiver_channel) = mpsc::channel();
            let (client_to_server_sender_channel, client_to_server_receiver_channel) = mpsc::channel();

            if self.clients.contains_key(&client_id) {
                println!("Client reconnected: {:?}", client_id);
            }
            else { 
                let new_client = Client::new(client_id, "PASSWORD".to_string(), stream, server_to_client_receiver_channel, client_to_server_sender_channel, true, 0);
                self.clients.insert(client_id, new_client);
            }

            self.create_new_client_thread(server_to_client_receiver_channel, client_to_server_sender_channel, stream);
            println!("New client connected: {:?}", client_id);
            self.active_connections.insert(client_id);
        }
    }

    pub fn create_new_client_thread(receiver_channel: std::sync::mpsc::Receiver<ClientTask>, sender_channel: std::sync::mpsc::Sender<TopicHandlerTask> ,mut stream: TcpStream) {
        thread::spawn(move || {
            let mut receiver_channel = receiver_channel;
            let mut sender_channel = sender_channel;

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
                        handle_packet(packet, address.clone(), stream);
    
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

pub fn handle_packet(packet: Packet, client_id: Vec<u8>, stream: TcpStream) {
    match packet {
        Packet::Publish(publish_packet)  => handle_publish(publish_packet, stream),
        Packet::Puback(puback_packet) => handle_puback(puback_packet, stream),
        Subscribe => handle_subscribe(packet, stream),
        Unsubscribe => handle_unsubscribe(packet, stream),
        Pingreq => handle_pingreq(packet, stream),
        Disconnect => handle_disconnect(packet, stream),
        _ => println!("Unsupported packet type"),
    }
}
pub fn handle_publish(packet: Packet, stream: TcpStream) {
    if (topic_handler.publish(packet)){
        send_puback(stream);
    } else {
        kill_thread(stream);
    } 
}


pub fn handle_subscribe(packet: Packet, stream: TcpStream) {
    if (topic_handler.subscribe(packet)){
        send_suback(stream);
    } else {
        kill_thread(stream);
    } 
}

pub fn handle_unsubscribe(packet: Packet, stream: TcpStream) {
    if (topic_handler.unsubscribe(packet)){
        send_unsuback(stream);
    } else {
        kill_thread(stream);
    } 
}

pub fn handle_disconnect(packet: Packet, stream: TcpStream) {
    kill_thread(stream);
}

pub fn handle_pingreq(packet: Packet, stream: TcpStream) {
    send_pingresp(stream);
}

pub fn send_pingresp(stream: TcpStream) {
    let pingresp = Pingresp::new();
    let pingresp_bytes = pingresp.into_bytes();
    stream.write_all(&pingresp_bytes);
}

pub fn send_puback(stream: TcpStream) {
    let puback = Puback::new();
    let puback_bytes = puback.into_bytes();
    stream.write_all(&puback_bytes);
}

pub fn send_suback(stream: TcpStream) {
    let suback = Suback::new();
    let suback_bytes = suback.into_bytes();
    stream.write_all(&suback_bytes);
}

pub fn send_unsuback(stream: TcpStream) {
    let unsuback = Unsuback::new();
    let unsuback_bytes = unsuback.into_bytes();
    stream.write_all(&unsuback_bytes);
}

pub fn kill_thread(stream: TcpStream) {
    // LOGICA DE MATAR EL THREAD ACTUAL
}



// Preguntar: ¿Cómo se maneja el cierre de la conexión?
// Cual es el alcance de cada client thread?
// Hilo Topic Handler

