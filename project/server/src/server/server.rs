pub struct Server {
    clients: HashMap<Vec<u8>, Client>,
    topic_handler: TopicHandler,
    //config: Config
}

impl Server{
    fn new() -> Self {
        Server {
            clients: Vec::new(),
            topic_handler: TopicHandler::new(),
        }
    }

    fn handle_packet(&self, packet: Packet, client_id: Vec<u8>) {
        match packet {
            Package::Connect => {
                self.clients.insert(client_id, Client::new(client_id));
                println!("Client connected: {:?}", client_id);
            },
            Packet::Publish(topic, message) => {
                if let Some(client) = self.clients.get(&client_id) {
                    self.topic_handler.publish(topic, message, client);
                    println!("Message published to topic: {:?}", topic);
                } else {
                    println!("Received publish from unknown client: {:?}", client_id);
                }
            },
            _ => println!("Unsupported packet type"),
        }
    } 


    fn wait_connection(&self, ) {
        let listener = TcpListener::bind(address).unwrap();
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let mut server = self.clone();

            thread::spawn(move || {
                let mut buffer = [0; 1024];
                loop {
                    let bytes_read = stream.read(&mut buffer).unwrap();

                    if bytes_read == 0 {
                        return;
                    }

                    let packet = Packet::from_bytes(&buffer[..bytes_read]).unwrap();
                    let client_id = packet.client_id().unwrap();

                    server.handle_packet(packet, client_id);
                }
            });
        }
    }

    
}