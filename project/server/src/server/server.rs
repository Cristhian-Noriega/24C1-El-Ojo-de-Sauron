#![allow(dead_code)]
use std::{
    collections::{HashMap, HashSet, VecDeque}, io::BufReader, net::{TcpListener, TcpStream}, sync::mpsc::{self, TryRecvError}, thread
};
use crate::{client::{Client, ClientTask}, config::Config, topic_handler::TopicHandler};

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
                    server.handle_packet(packet, address, stream);
                }
                Err(err) => {
                    println!("Error al recibir paquete: {:?}", err);
                }
            }
        }

        Ok(())
    }

    pub fn handle_packet(&self, packet: Packet, client_id: Vec<u8>, stream: TcpStream) {
        match packet {
            Connect => self.handle_connect(packet, stream),
            Publish => self.handle_publish(packet),
            Puback => self.handle_puback(packet),
            Subscribe => self.handle_subscribe(packet),
            Unsubscribe => self.handle_unsubscribe(packet),
            Pingreq => self.handle_pingreq(packet),
            Disconnect => self.handle_disconnect(packet),
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
                
                self.create_new_client_thread(new_client, receiver_channel);
                self.clients.insert(client_id, new_client);
                println!("New client connected: {:?}", client_id);
            }
            self.clients.get(&client_id).send_task(ClientTask::SendConnack);
            self.active_connections.insert(client_id);
        }
    }

    pub fn handle_publish(&self, packet: Packet) {
        // let topic = packet.topic().unwrap();
        // let message = packet.message().unwrap();
        // let client_id = packet.client_id().unwrap();
        // let packet = Packet::PublishPacket::new(client_id, topic, message);

        self.topic_handler.publish(packet);
    }

    pub fn handle_subscribe(&self, packet: Packet) {
        let client_id = packet.client_id().unwrap();
        let topic = packet.topic().unwrap();
        let qos = packet.qos().unwrap();

        if let Some(client) = self.clients.get(&client_id) {
            client.subscribe(topic, qos);
            client.send_task(ClientTask::send_suback);
        } else {
            println!("Failed to subscribe unknown client: {:?}", client_id);
        }
    }

    pub fn handle_unsubscribe(&self, packet: Packet) {
        let client_id = packet.client_id().unwrap();
        let topic = packet.topic().unwrap();

        if let Some(client) = self.clients.get(&client_id) {
            client.unsubscribe(topic);
            client.send_task(ClientTask::send_unsuback);
        } else {
            println!("Failed to unsubscribe unknown client: {:?}", client_id);
        }
    }

    pub fn handle_pingreq(&self, packet: Packet) {
        let client_id = packet.client_id().unwrap();

        if let Some(client) = self.clients.get(&client_id) {
            client.send_task(ClientTask::send_pingresp);
        } else {
            println!("Failed to send pingresp to unknown client: {:?}", client_id);
        }
    }

    pub fn handle_disconnect(&self, packet: Packet) {
        let client_id = packet.client_id().unwrap();
        self.active_connections.remove(&client_id);
        self.clients.remove(&client_id);
        // TO DO: MATAR THREAD DEL CLIENTE
    }

    pub fn create_new_client_thread(&self, client: Client, receiver_channel: std::sync::mpsc::Receiver<ClientTask>) {
        thread::spawn(move || {
            let mut current_tasks: VecDeque<ClientTask> = VecDeque::new();
            loop {
                match receiver_channel.try_recv() {
                    Ok(task) => current_tasks.push_back(task),
                    Err(TryRecvError::Empty) => (),
                    Err(TryRecvError::Disconnected) => break,
                }

                while let Some(task) = current_tasks.pop_front() {
                    match task {
                        SendConnack => client.stream_packet(Connack),
                    }
                }
            }
        });
    }

    pub fn stream_packet(&self, packet: Packet, client_id: Vec<u8>) {
        if let Some(client) = self.clients.get(&client_id) {
            client.stream_packet(packet);
        } else {
            println!("Failed to send packet to unknown client: {:?}", client_id);
        }
    }
}
