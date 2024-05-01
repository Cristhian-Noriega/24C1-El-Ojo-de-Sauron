// Servicio de mensajería
// Independientemente del protocolo elegido se recomienda seguir el patrón de comunicación publisher-suscriber y 
//la arquitectura cliente-servidor. Para lo cual se deberá implementar por un lado el servidor de mensajería y 
// por otro lado una library que permitirá la comunicación por parte de los clientes. Se deberá tener en cuenta los siguientes requerimientos:

// Seguridad: autenticación, autorización, encriptación, etc.
// Calidad de servicio (Quality of Service, QoS): como mínimo se debe soportar 'at least one'
// Reliability: el servidor deberá tener registro de los clientes conectados y permitir a un cliente que sufra una desconexión 
// poder reconectarse y obtener los mensajes que no recibió (sesiones y almacenamiento de mensajes)
// Configuración: debe incluir todos los parámetros necesarios para la ejecución del servidor, como el puerto, direccion IP, etc.
// (no esta permitido definir estos valores mediante constantes en el código)
// Logging: debe registrar un resumen de los mensajes recibidos y enviados, y cualquier error o evento significativo durante la 
// ejecucion del servidor.

pub struct Server {
    clients: HashMap<Client_id, Client>,
    active_connections: Vec<Connection>,
    topic_handler: TopicHandler,
    config: Config
}

//THREAD-POOL OR THREAD-PER-CONNECTION

impl Server{
    fn new() -> Self {
        Server {
            clients: Vec::new(),
            topic_handler: TopicHandler::new(),
        }
    }

    fn handle_package(&self, package: package, client_id: Vec<u8>) {
        match package {
            Package::Connect => {
                self.clients.insert(client_id, Client::new(client_id));
                println!("Client connected: {:?}", client_id);
            },
            Package::Publish(topic, message) => {
                if let Some(client) = self.clients.get(&client_id) {
                    self.topic_handler.publish(topic, message, client);
                    println!("Message published to topic: {:?}", topic);
                } else {
                    println!("Received publish from unknown client: {:?}", client_id);
                }
            },
            Package::Subscribe(topic) => {
                if let Some(client) = self.clients.get(&client_id) {
                    client.subscribe(topic);
                    println!("Client subscribed to topic: {:?}", topic);
                } else {
                    println!("Received subscribe from unknown client: {:?}", client_id);
                }
            },
            Package::Unsubscribe(topic) => {
                if let Some(client) = self.clients.get(&client_id) {
                    client.unsubscribe(topic);
                    println!("Client unsubscribed from topic: {:?}", topic);
                } else {
                    println!("Received unsubscribe from unknown client: {:?}", client_id);
                }
            },
            _ => println!("Unsupported package type"),
        }
    }

    fn send_package(&self, package: Package, client_id: Vec<u8>) {
        if let Some(client) = self.clients.get(&client_id) {
            client.send_package(package);
        } else {
            println!("Failed to send package to unknown client: {:?}", client_id);
        }
    }

    fn send_connack(&self, client_id: Vec<u8>) {
        let connack = Package::Connack;
        self.send_package(connack, client_id);
    }

    fn send_puback(&self, client_id: Vec<u8>) {
        let puback = Package::Puback;
        self.send_package(puback, client_id);
    }

    fn send_suback(&self, client_id: Vec<u8>) {
        let suback = Package::Suback;
        self.send_package(suback, client_id);
    }

    fn send_unsuback(&self, client_id: Vec<u8>) {
        let unsuback = Package::Unsuback;
        self.send_package(unsuback, client_id);
    }

    fn send_pingresp(&self, client_id: Vec<u8>) {
        let pingresp = Package::Pingresp;
        self.send_package(pingresp, client_id);
    }

    fn send_disconnect(&self, client_id: Vec<u8>) {
        let disconnect = Package::Disconnect;
        self.send_package(disconnect, client_id);
    }

    fn wait_connection(&self, ) {
        let listener = TcpListener::bind(address).unwrap();
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let mut server = self.clone();

            let mut buffer = [0; 1024];
            let bytes_read = stream.read(&mut buffer).unwrap();

            if bytes_read == 0 {
                return;
            }

            let package = Package::from_bytes(&buffer[..bytes_read]).unwrap();
            let client_id = package.client_id().unwrap();

            server.handle_package(package, client_id);
        }
    }
}
