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

// TODO: move to file with users and passwords
const ADMIN_ID: &[u8] = b"admin";
const CAMERA_SYSTEM_ID: &[u8] = b"camera-system";

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
}

impl ClientManager {
    /// Creates a new client manager with an empty Clients map
    pub fn new() -> Self {
        Self {
            registered_clients: Arc::new(Mutex::new(Clients::new())),
        }
    }

    /// Registers a client with the specified client ID, username, and password
    pub fn register_client(&self, client_id: Vec<u8>, username: Vec<u8>, password: Vec<u8>) {
        let mut registered_clients = self.registered_clients.lock().unwrap();
        registered_clients.insert(client_id, (username, password, false));
    }

    /// Authenticates a client with the specified client ID, username, and password
    pub fn authenticate_client(
        &self,
        client_id: Vec<u8>,
        username: Vec<u8>,
        password: Vec<u8>,
    ) -> bool {
        let mut registered_clients = self.registered_clients.lock().unwrap();
        if let Some((stored_username, stored_password, is_connected)) =
            registered_clients.get_mut(&client_id)
        {
            if stored_username == &username && stored_password == &password {
                *is_connected = true;
                return true;
            }
        }
        false
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

        if self.authenticate_client(client_id.clone(), username, password) {
            Some(Client::new(
                client_id.clone(),
                stream.try_clone().unwrap(),
                true,
                0,
            ))
        } else {
            self.failure_connection(stream, ConnectReturnCode::IdentifierRejected, key);
            None
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
    pub fn get_login_info(&self, connect_packet: &Connect) -> Result<(Vec<u8>, Vec<u8>), String> {
        let login = connect_packet
            .login()
            .ok_or("No login information provided")?;
        let username = login.username().content().to_vec();
        let password = login
            .password()
            .ok_or("No password provided")?
            .content()
            .to_vec();
        Ok((username, password))
    }

    /// Makes the initial registrations for the admin and camera system clients
    pub fn make_initial_registrations(&self, config: Config) {
        let admin_username = config.get_admin_username().as_bytes().to_vec();
        let admin_password = config.get_admin_password().as_bytes().to_vec();
        let admin_id = ADMIN_ID.to_vec();

        self.register_client(admin_id, admin_username, admin_password);

        let camera_system_username = config.get_camera_system_username().as_bytes().to_vec();
        let camera_system_password = config.get_camera_system_password().as_bytes().to_vec();
        let camera_system_id = CAMERA_SYSTEM_ID.to_vec();

        self.register_client(
            camera_system_id,
            camera_system_username,
            camera_system_password,
        );
    }
}
