use std::{
    collections::HashMap,
    io::Write,
    net::TcpStream,
    sync::{Arc, Mutex},
};

use mqtt::model::{
    packets::{connack::Connack, connect::Connect},
    return_codes::connect_return_code::ConnectReturnCode,
};

use crate::{client::Client, config::Config};

const ADMIN_USERNAME: &[u8] = b"admin";

type Logins = HashMap<(Vec<u8>, Vec<u8>), bool>; // key: (username, password), value: is_connected

#[derive(Debug, Clone)]
pub struct ClientManager {
    registered_clients: Arc<Mutex<Logins>>,
    // log_file: Arc<crate::logfile::Logger>,
}

impl ClientManager {
    pub fn new() -> Self {
        Self {
            registered_clients: Arc::new(Mutex::new(Logins::new())),
        }
    }

    pub fn register_client(&self, username: Vec<u8>, password: Vec<u8>) {
        let mut clients = self.registered_clients.lock().unwrap();
        clients.insert((username, password), false);
    }

    pub fn authenticate_client(&self, username: Vec<u8>, password: Vec<u8>) -> bool {
        let registered_clients = self.registered_clients.lock().unwrap();
        registered_clients.contains_key(&(username, password))
    }

    pub fn process_connect_packet(
        &self,
        connect_packet: Connect,
        stream: TcpStream,
    ) -> Option<Client> {
        let client_id = connect_packet.client_id().content();
        let (usermame, password) = match connect_packet.login() {
            Some(login) => {
                let username = login.username().content();
                match login.password() {
                    Some(password) => {
                        let password = password.content();
                        (username, password)
                    }
                    None => {
                        //self.log_file.error("No password provided");
                        self.failure_connection(stream, ConnectReturnCode::IdentifierRejected);
                        return None;
                    }
                }
            }
            None => {
                //self.log_file.error("No login information provided");
                self.failure_connection(stream, ConnectReturnCode::IdentifierRejected);
                return None;
            }
        };

        let mut registered_clients = match self.registered_clients.lock() {
            Ok(registered_clients) => registered_clients,
            Err(err) => {
                // self.log_file
                // .error(&format!("Error locking registered clients: {:?}", err));
                return None;
            }
        };

        match registered_clients.get(&(usermame.to_vec(), password.to_vec())) {
            Some(true) => {
                //self.log_file.error("Client already connected");
                self.failure_connection(stream, ConnectReturnCode::IdentifierRejected);
                return None;
            }
            Some(false) => {
                registered_clients.insert((usermame.to_vec(), password.to_vec()), true);
            }
            None => {
                //self.log_file.error("Client not registered");
                self.failure_connection(stream, ConnectReturnCode::IdentifierRejected);
                return None;
            }
        }

        Some(Client::new(
            client_id.clone(),
            stream.try_clone().unwrap(),
            true,
            0,
        ))
    }

    fn failure_connection(&self, mut stream: TcpStream, return_code: ConnectReturnCode) {
        let connack = Connack::new(false, return_code);
        let connack_bytes = connack.to_bytes();

        match stream.write_all(&connack_bytes) {
            Ok(_) => {}
            Err(err) => {
                println!("Error sending Connack packet: {:?}", err);
            }
        }
    }

    pub fn make_initial_registrations(&self, config: Config) {
        let mut registered_clients = self.registered_clients.lock().unwrap();
        registered_clients.insert(
            (
                ADMIN_USERNAME.to_vec(),
                config.get_admin_password().as_bytes().to_vec(),
            ),
            false,
        );
        registered_clients.insert(
            (
                config.get_camera_system_username().as_bytes().to_vec(),
                config.get_camera_system_password().as_bytes().to_vec(),
            ),
            false,
        );
    }
}