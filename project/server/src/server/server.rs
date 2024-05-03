use std::env::args;
use std::io::{BufRead, BufReader, Read};
use std::net::TcpListener;
use std::thread;

static SERVER_ARGS: usize = 2;

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
    active_connections: HashSet<Connection>,
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

pub enum ClientTask{
    send_connack,
    send_publish,
    send_puback,
    send_suback,
    send_unsuback,
    send_pingresp,
    send_suback,
}

//THREAD-PER-CONNECTION

//TO DO:

// Cris::do( mergear archivos server.rs y client.rs en carpeta server
// seguir implementacion del server  
// armar el server que escuche conexiones y cree un thread por cada cliente que se conecta
// Ver logica handle_client
// considerar matar o no el thread del client desconectado 



// Mate::do(
// handlear recepción de paquetes y emisión de paquetes. Implementar cada acción que se realiza cuando el servidor envia 
// cada posible paquete y cuando lo recibe.)
// Connect: si no existe lo crea y conecta, si ya existe solo conecta. Una vez que termina mandar Connack.
// Connack: enviar paquete por el stream del cliente. 
// Publish: Cliente publica un mensaje en topico "A" => pasar al topicHandler ese mensaje con ese topico => el TopicHandler 
// crea los paquetes y los ids y los publica a los clientes suscriptos en ese instante. Pensar mas esta logica.
// Subscribe: => 
// Un cliente está conectado. cómo saber si perdió la conexión? Pings?
// Considerar para ese caso, el field alive del estado del cliente, usando un AtomicBool para que sea thread safe.
// Hay un thread por cada cliente. Ese thread cada un determinado tiempo manda un ping para ver si está conectado.
// El ping lo manda solo si en ese tiempo el cliente no hizo nada. Si considera estar desconectado, sacarlo de las active_connections
// Mata el thread. Qué hace el TopicHandler ahí??? Pasarle a TopicHandler las activeConnection para que sepa decidir eso
// TopicHandler: mandarles los PubAck
// 
//    );    

// CLIENT TO SERVER:
// Implementación de CONNECT -> Refinar modelo 
// Implementación de PUBLISH 
// Implementación de PUBACK
// Implementación de SUBSCRIBE
// Implementación de UNSUBSCRIBE
// Implementación de PINGREQ
// Implementación de DISCONNECT

// SERVER TO CLIENT:
// Implementación de CONNACK 
// Implementación de PUBLISH
// Implementación de PUBACK
// Implementación de SUBACK
// Implementación de UNSUBACK
// Implementación de PINGRESP


fn main() -> Result<(), ()> {
    let argv = args().collect::<Vec<String>>();
    if argv.len() != SERVER_ARGS {
        println!("Cantidad de argumentos inválido");
        let app_name = &argv[0];
        println!("Usage:\n{:?} <puerto>", app_name);
        return Err(());
    }

    let address = "127.0.0.1:".to_owned() + &argv[1]; // HARDCODEADO
    if let Err(err) = server_run(&address) {
        println!("Error al ejecutar el servidor: {:?}", err);
        return Err(());
    }

    Ok(())
}

fn server_run(address: &str) -> std::io::Result<()> {
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
                server::handle_packet(packet);
            }
            Err(err) => {
                println!("Error al recibir paquete: {:?}", err);
            }
        }
    }

    Ok(())
}

impl Server{
    fn new() -> Self {
        Server {
            clients: Vec::new(),
            topic_handler: TopicHandler::new(),
        }
    }

    fn handle_packet(&self, packet: packet, client_id: Vec<u8>) {
        match packet {
            packet::Connect => self.handle_connect(packet),
            packet::Publish => self.handle_publish(packet),
            packet::Puback => self.handle_puback(packet),
            packet::Subscribe => self.handle_subscribe(packet),
            packet::Unsubscribe => self.handle_unsubscribe(packet),
            packet::Pingreq => self.handle_pingreq(packet),
            packet::Disconnect => self.handle_disconnect(packet),
            _ => println!("Unsupported packet type"),
        }
    }

    fn handle_connect(&self, packet: packet::ConnectPacket) {
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
                let new_client = Client::new(client_id, sender_channel, packet);
                
                self.create_new_client_thread(new_client, receiver_channel);
                self.clients.insert(client_id, new_client);
                println!("New client connected: {:?}", client_id);
            }
            self.clients.get(&client_id).send_task(ClientTask::send_connack);
            self.active_connections.insert(client_id);
        }
    }

    fn handle_publish(&self, packet: packet::PublishPacket) {
        let topic = packet.topic().unwrap();
        let message = packet.message().unwrap();
        let client_id = packet.client_id().unwrap();
        let packet = packet::PublishPacket::new(client_id, topic, message);

        self.topic_handler.publish(packet);
        client.send_task(ClientTask::send_puback);
    }

    fn handle_subscribe(&self, packet: packet::SubscribePacket) {
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

    fn handle_unsubscribe(&self, packet: packet::UnsubscribePacket) {
        let client_id = packet.client_id().unwrap();
        let topic = packet.topic().unwrap();

        if let Some(client) = self.clients.get(&client_id) {
            client.unsubscribe(topic);
            client.send_task(ClientTask::send_unsuback);
        } else {
            println!("Failed to unsubscribe unknown client: {:?}", client_id);
        }
    }

    fn handle_pingreq(&self, packet: packet::PingreqPacket) {
        let client_id = packet.client_id().unwrap();

        if let Some(client) = self.clients.get(&client_id) {
            client.send_task(ClientTask::send_pingresp);
        } else {
            println!("Failed to send pingresp to unknown client: {:?}", client_id);
        }
    }

    fn handle_disconnect(&self, packet: packet::DisconnectPacket) {
        let client_id = packet.client_id().unwrap();
        active_connections.remove(&client_id);
        clients.remove(&client_id);
        // TO DO: MATAR THREAD DEL CLIENTE
    }

    fn create_new_client_thread(&self, new_client: Client, receiver_channel: std::sync::mpsc::Receiver<ClientTask>) {
        thread::spawn(move || {
            let mut current_tasks: VecDeque<ClientTask> = VecDeque::new();
            loop {
                match receiver_channel.try_recv() {
                    Ok(task) => current_tasks.push_back(task),
                    Err(TryRecvError::Empty) => (),
                    Err(TryRecvError::Disconnected) => break,
                }

                while let Some(task) = tasks.pop_front() {
                    match task {
                        send_suback => client.stream_suback(),
                    }
                }
            }
        });
    }

    fn stream_packet(&self, packet: packet, client_id: Vec<u8>) {
        if let Some(client) = self.clients.get(&client_id) {
            client.stream_packet(packet);
        } else {
            println!("Failed to send packet to unknown client: {:?}", client_id);
        }
    }
}
