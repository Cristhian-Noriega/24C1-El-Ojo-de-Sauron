#![allow(dead_code)]
use std::{
    collections::{HashMap, HashSet, VecDeque}, io::BufReader, net::{Shutdown, TcpListener, TcpStream}, sync::mpsc::{self, TryRecvError}, thread
};
use crate::{client::{Client, ClientTask}, config::Config, topic_handler::{self, TopicHandler}};

use crate::model::packets::{Connect, Connack, Publish, Puback, Subscribe, Suback, Unsubscribe, Unsuback, Pingreq, Pingresp, Disconnect};

pub struct Server {
    clients: HashMap<String, Client>,
    active_connections: HashSet<i32>,
    topic_handler: TopicHandler,
    config: Config
}

pub enum Packet {
    Connect(Connect),
    Connack(Connack),
    Publish(Publish),
    Puback(Puback),
    Subscribe(Subscribe),
    Suback(Suback),
    Unsubscribe(Unsubscribe),
    Unsuback(Unsuback),
    Pingreq(Pingreq),
    Pingresp(Pingresp),
    Disconnect(Disconnect),
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
            Connect => self.handle_connect(packet, stream),
            // Publish => self.handle_publish(packet),
            // Puback => self.handle_puback(packet),
            // Subscribe => self.handle_subscribe(packet),
            // Unsubscribe => self.handle_unsubscribe(packet),
            // Pingreq => self.handle_pingreq(packet),
            // Disconnect => self.handle_disconnect(packet),
            _ => println!("Unsupported packet type"),
        }
    }

    pub fn handle_connect(&self, packet: Packet, stream: TcpStream) {
        let client_id = packet.client_id().unwrap();

        if self.active_connections.contains(&client_id) {
            println!("Client already connected: {:?}", client_id);
            return;
        }
        else {
            if self.clients.contains_key(&client_id) {
                println!("Client reconnected: {:?}", client_id);
            }
            else { // CLIENTE NUEVO (CREAR)
                let (sender_channel, receiver_channel) = mpsc::channel();
                let new_client = Client::new(client_id, "PASSWORD".to_string(), stream, sender_channel, true, 0);
                
                self.create_new_client_thread(receiver_channel, stream);
                self.clients.insert(client_id, new_client);
                println!("New client connected: {:?}", client_id);
            }
            self.active_connections.insert(client_id);
        }
    }

    // pub fn handle_publish(&self, packet: Packet) {
    //     self.topic_handler.publish(packet);
    // }

    // pub fn handle_subscribe(&self, packet: Packet) {
    //     let client_id = packet.client_id().unwrap();
    //     let topic = packet.topic().unwrap();
    //     let qos = packet.qos().unwrap();

    //     if let Some(client) = self.clients.get(&client_id) {
    //         client.subscribe(topic, qos);
    //         client.send_task(ClientTask::send_suback);
    //     } else {
    //         println!("Failed to subscribe unknown client: {:?}", client_id);
    //     }
    // }

    // pub fn handle_unsubscribe(&self, packet: Packet) {
    //     let client_id = packet.client_id().unwrap();
    //     let topic = packet.topic().unwrap();

    //     if let Some(client) = self.clients.get(&client_id) {
    //         client.unsubscribe(topic);
    //         client.send_task(ClientTask::send_unsuback);
    //     } else {
    //         println!("Failed to unsubscribe unknown client: {:?}", client_id);
    //     }
    // }

    // pub fn handle_pingreq(&self, packet: Packet) {
    //     let client_id = packet.client_id().unwrap();

    //     if let Some(client) = self.clients.get(&client_id) {
    //         client.send_task(ClientTask::send_pingresp);
    //     } else {
    //         println!("Failed to send pingresp to unknown client: {:?}", client_id);
    //     }
    // }

    // pub fn handle_disconnect(&self, packet: Packet) {
    //     let client_id = packet.client_id().unwrap();
    //     self.active_connections.remove(&client_id);
    //     self.clients.remove(&client_id);
    //     // TO DO: MATAR THREAD DEL CLIENTE
    // }

    pub fn create_new_client_thread(receiver_channel: std::sync::mpsc::Receiver<ClientTask>, mut stream: TcpStream) {
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
    
                        // Handle the received packet
                        println!("Packet received from address: {:?}", address);
                        handle_packet(packet, address.clone(), stream);
    
                        // Clear the buffer for the next read
                        buffer.clear();
                    }
                    Err(err) => {
                        // Handle read errors, e.g., connection reset by peer
                        println!("Error reading from stream: {:?}", err);
                        break; // Exit the loop and terminate the thread
                    }
                }
            }
        });
    }
    
    
}

pub fn handle_packet(packet: Packet, client_id: Vec<u8>, stream: TcpStream) {
    match packet {
        Publish => handle_publish(packet, stream),
        Puback => handle_puback(packet, stream),
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