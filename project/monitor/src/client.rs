use std::env::args;
use std::io::Read;
use std::net::TcpStream;
use std::io::Write;
use sauron::model::components::encoded_string::EncodedString;
pub use sauron::model::{
    packet::Packet,
    packets::connect::Connect,
};

static CLIENT_ARGS: usize = 3;

pub struct Client {
    pub connection_status: String,
    pub response_text: String,
    pub address: String,
}

impl Client {
    pub fn new() -> Self {
        let argv = args().collect::<Vec<String>>();
        if argv.len() != CLIENT_ARGS {
            let app_name = &argv[0];
            println!("{:?} <host> <puerto>", app_name);
        }

        let address = argv[1].clone() + ":" + &argv[2];

        Self {
            connection_status: "offline".to_owned(),
            response_text: "no response".to_owned(),
            address,
        }
    }

    pub fn connect_to_server(&self) -> std::io::Result<TcpStream> {
        println!("Conectándome a {:?}", self.address);
        let mut to_server_stream = TcpStream::connect(&self.address)?;
    
        let client_id_bytes: Vec<u8> = b"monitor".to_vec();
        let client_id = EncodedString::new(client_id_bytes);
        let will = None;
        let login = None;
        let connect_package = Connect::new(false, 0, client_id, will, login);
    
        let _ = to_server_stream.write(connect_package.to_bytes().as_slice());
    
        Ok(to_server_stream)
    }

    pub fn client_run(&mut self) -> std::io::Result<()> {
        let mut to_server_stream = self.connect_to_server()?;

        let mut buffer = [0; 1024];
        let _ = to_server_stream.read(&mut buffer);
        let packet = Packet::from_bytes(&mut buffer.as_slice()).unwrap();

        match packet {
            Packet::Connack(connack) => {
                println!(
                    "Received Connack packet with return code: {:?} and sessionPresent: {:?}",
                    connack.connect_return_code(),
                    connack.session_present()
                );
                self.connection_status = "connected".to_string();
                self.response_text = format!("{:?}", connack);
            }
            _ => println!("Received unsupported packet type"),
        }

        Ok(())
    }
}