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
        match self.registered_clients.lock() {
            Ok(mut registered_clients) => {
                registered_clients.insert((username, password), false);
            }
            Err(_) => {
                // self.log_file.error("Error locking registered clients");
            }
        }
    }

    pub fn authenticate_client(&self, username: Vec<u8>, password: Vec<u8>) -> bool {
        match self.registered_clients.lock() {
            Ok(registered_clients) => match registered_clients.get(&(username, password)) {
                Some(true) => true,
                _ => false,
            },
            Err(_) => {
                // self.log_file.error("Error locking registered clients");
                false
            }
        }
    }

    pub fn process_connect_packet(
        &self,
        connect_packet: Connect,
        stream: TcpStream,
    ) -> Option<Client> {
        let client_id = connect_packet.client_id().content();
        let (usermame, password) = match self.get_login_info(&connect_packet) {
            Ok((username, password)) => (username, password),
            Err(_) => {
                // self.log_file.error(&format!("Error getting login info: {:?}", err));
                return None;
            }
        };

        let mut registered_clients = match self.registered_clients.lock() {
            Ok(registered_clients) => registered_clients,
            Err(_) => {
                // self.log_file
                // .error(&format!("Error locking registered clients: {:?}", err));
                return None;
            }
        };

        // if it is not registered i have to failure the connection
        //i have to check if the client is already connected , with the bool value,
        //if it is connected i have to return None
        // if it is registered and is not connected i have to change the value to true

        match registered_clients.get_mut(&(usermame.clone(), password.clone())) {
            Some(is_connected) => {
                if *is_connected {
                    println!("Client already connected");
                    self.failure_connection(stream, ConnectReturnCode::IdentifierRejected);
                    return None;
                }
                *is_connected = true;
                println!("Client registered now is connected")
            }
            None => {
                println!("Client not registered");
                self.failure_connection(stream, ConnectReturnCode::BadUsernameOrPassword);
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

    pub fn get_login_info(&self, connect_packet: &Connect) -> Result<(Vec<u8>, Vec<u8>), String> {
        let login = connect_packet
            .login()
            .ok_or("No login information provided")?;
        let username = login.username().content();
        let password = login.password().ok_or("No password provided")?.content();
        Ok((username.to_vec(), password.to_vec()))
    }

    pub fn make_initial_registrations(&self, config: Config) {
        let admin_username = ADMIN_USERNAME.to_vec();
        let admin_password = config.get_admin_password().as_bytes().to_vec();

        self.register_client(admin_username, admin_password);

        let camera_system_username = config.get_camera_system_username().as_bytes().to_vec();
        let camera_system_password = config.get_camera_system_password().as_bytes().to_vec();

        self.register_client(camera_system_username, camera_system_password);
    }
}
