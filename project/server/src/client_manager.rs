use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::Write,
    net::TcpStream,
    sync::{
        mpsc::{self, Sender},
        Arc, Mutex,
    },
    thread,
};

use mqtt::model::{
    packets::{connack::Connack, connect::Connect},
    return_codes::connect_return_code::ConnectReturnCode,
};

use crate::{client::Client, error::{ServerError, ServerResult}};

/// Represents a client ID
type ClientId = Vec<u8>;
/// Represents a tuple of a username, password, and whether the client is connected
type Logins = (Vec<u8>, Vec<u8>, bool); // username, password, is_connected
/// Represents a map of client IDs to login information
type Clients = HashMap<ClientId, Logins>;

/// Represents a manager that handles clients in the server such as registering and authenticating them
/// and processing connect packets validating the login information
#[derive(Debug, Clone)]
pub struct ClientManager {
    registered_clients: Arc<Mutex<Clients>>,
    file_sender: Sender<String>,
}

impl ClientManager {
    /// Creates a new client manager with an empty Clients map
    pub fn new(login_file_path: &str) -> Self {
        let (sender, receiver) = mpsc::channel();
        let file_path = login_file_path.to_string();

        let registered_clients = Self::intials_registers(&file_path);

        thread::spawn(move || {
            let mut file = match OpenOptions::new()
                .create(true)
                .append(true)
                .open(&file_path)
            {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("Failed to open login file: {}", e);
                    return;
                }
            };

            for log_entry in receiver {
                if let Err(e) = writeln!(file, "{}", log_entry) {
                    eprintln!("Failed to write to login file: {}", e);
                }
            }
        });

        Self {
            registered_clients: Arc::new(Mutex::new(registered_clients)),
            file_sender: sender,
        }
    }

    /// Registers a client with the specified client ID, username, and password
    pub fn register_client(&self, client_id: Vec<u8>, username: Vec<u8>, password: Vec<u8>) -> ServerResult<()> {
        self.add_client(client_id.clone(), username.clone(), password.clone())?;
        self.save_client(client_id, username, password)?;

        Ok(())
    }

    /// Adds a client to the registered clients map
    fn add_client(&self, client_id: Vec<u8>, username: Vec<u8>, password: Vec<u8>) -> ServerResult<()>  {
        let mut registered_clients = self.registered_clients.lock()?;
        registered_clients.insert(
            client_id.clone(),
            (username.clone(), password.clone(), false),
        );

        Ok(())
    }

    /// Saves a client to the login file
    fn save_client(&self, client_id: Vec<u8>, username: Vec<u8>, password: Vec<u8>) -> ServerResult<()>{
        let client_id = String::from_utf8(client_id)?;

        let username =  String::from_utf8(username)?;

        let password = String::from_utf8(password)?;

        let login_entry = format!("{} = {} = {}", client_id, username, password);

        self.file_sender.send(login_entry)?;

        Ok(())
    }

    /// Authenticates a client with the specified client ID, username, and password
    pub fn authenticate_client(
        &self,
        client_id: Vec<u8>,
        username: Vec<u8>,
        password: Vec<u8>,
    ) -> ServerResult<bool> {
        let mut registered_clients =  self.registered_clients.lock()?;

        if let Some((stored_username, stored_password, is_connected)) =
            registered_clients.get_mut(&client_id)
        {
            if *is_connected {
                return Ok(false);
            }

            if stored_username == &username && stored_password == &password {
                *is_connected = true;
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Disconnects a client with the specified client ID
    pub fn disconnect_client(&self, client_id: Vec<u8>) -> ServerResult<()> {
        let mut registered_clients = self.registered_clients.lock()?;
        if let Some((_, _, is_connected)) = registered_clients.get_mut(&client_id) {
            *is_connected = false;
        }
        
        Ok(())
    }

    /// Processes a connect packet by validating the login information and authenticating the client
    pub fn process_connect_packet(
        &self,
        connect_packet: Connect,
        stream: TcpStream,
        key: &[u8],
    ) -> Option<Client> {
        let client_id = connect_packet.client_id().content().to_vec();
        let (username, password) = match self.get_login_info(&connect_packet) {
            Ok(login) => login,
            Err(_) => {
                self.failure_connection(stream, ConnectReturnCode::BadUsernameOrPassword, key);
                return None;
            }
        };

        match self.authenticate_client(client_id.clone(), username, password) {
            Ok(true) => {
                let stream = match stream.try_clone() {
                    Ok(stream) => stream,
                    Err(err) => {
                        println!("Error cloning stream: {:?}", err);
                        return None;
                    }
                };
                Some(Client::new(client_id.clone(), Some(stream), true, 0))
            }

            Ok(false) => {
                self.failure_connection(stream, ConnectReturnCode::IdentifierRejected, key);
                None
            }
            Err(err) => {
                println!("Error authenticating client: {:?}", err);
                None
            }

        // if self.authenticate_client(client_id.clone(), username, password) {
        //     let stream = match stream.try_clone() {
        //         Ok(stream) => stream,
        //         Err(err) => {
        //             println!("Error cloning stream: {:?}", err);
        //             return None;
        //         }
        //     };
        //     Some(Client::new(client_id.clone(), Some(stream), true, 0))
        // } else {
        //     self.failure_connection(stream, ConnectReturnCode::IdentifierRejected, key);
        //     None

        }
    }

    /// Handles a failed connection by sending a Connack packet with the specified return code
    fn failure_connection(
        &self,
        mut stream: TcpStream,
        return_code: ConnectReturnCode,
        key: &[u8],
    ) {
        let connack = Connack::new(false, return_code);
        let connack_bytes = connack.to_bytes(key);

        if let Err(err) = stream.write_all(&connack_bytes) {
            println!("Error sending Connack packet: {:?}", err);
        }
    }

    /// Gets the login information from a connect packet
    pub fn get_login_info(&self, connect_packet: &Connect) -> ServerResult<(Vec<u8>, Vec<u8>)> {
        let login = connect_packet
            .login()
            .ok_or(ServerError::NoLoginProvided)?;
        let username = login.username().content().to_vec();
        let password = login
            .password()
            .ok_or(ServerError::NoPasswordProvided)?
            .content()
            .to_vec();
        Ok((username, password))
    }

    /// Makes the initial registrations reading the configuration file
    fn intials_registers(path: &str) -> HashMap<ClientId, Logins> {
        let content = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(_) => return HashMap::new(),
        };

        let mut registered_clients = HashMap::new();

        for line in content.lines() {
            let parts: Vec<&str> = line.split('=').map(|s| s.trim()).collect();
            if parts.len() == 3 {
                let client_id = parts[0].as_bytes().to_vec();
                let username = parts[1].as_bytes().to_vec();
                let password = parts[2].as_bytes().to_vec();
                registered_clients.insert(client_id, (username, password, false));
            }
        }

        registered_clients
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_client() {
        let client_manager = ClientManager::new("test_login_file.txt");
        let client_id = b"client1".to_vec();
        let username = b"username".to_vec();
        let password = b"password".to_vec();

        let _ = client_manager.register_client(client_id.clone(), username.clone(), password.clone());

        let registered_clients = client_manager.registered_clients.lock().unwrap();
        let logins = registered_clients.get(&client_id).unwrap();
        assert_eq!(logins.0, username);
        assert_eq!(logins.1, password);
        assert!(!logins.2);
    }
    

    #[test]
    fn test_authenticate_client() {
        let client_manager = ClientManager::new("test_login_file.txt");
        let client_id = b"client1".to_vec();
        let username = b"username".to_vec();
        let password = b"password".to_vec();

        let _ = client_manager.register_client(client_id.clone(), username.clone(), password.clone());

        assert!(client_manager.authenticate_client(
            client_id.clone(),
            username.clone(),
            password.clone()
        ).unwrap());
        assert!(!client_manager.authenticate_client(
            client_id.clone(),
            b"wrong".to_vec(),
            password.clone()
        ).unwrap());
        assert!(!client_manager.authenticate_client(
            client_id.clone(),
            username.clone(),
            b"wrong".to_vec()
        ).unwrap());
    }
}
