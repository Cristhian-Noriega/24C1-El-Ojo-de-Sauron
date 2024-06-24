#![allow(dead_code)]

use std::{fs::File, io::Read, path::Path};
use common::coordenate::Coordenate;
use serde_derive::{Deserialize, Serialize};

/// Represents the configuration of the server
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    address: String,
    key: [u8; 32],
    id: String,
    username: String,
    password: String,
    charging_stations: Vec<Coordenate>,
}

impl Config {
    /// Reads the configuration from a file
    pub fn from_file(path: &Path) -> std::io::Result<Self> {
        let mut file = File::open(path)?;
        let mut contents = String::new();

        file.read_to_string(&mut contents)?;

        let config = serde_json::from_str(&contents)?;

        Ok(config)
    }

    /// Returns the address of the server
    pub fn get_address(&self) -> &str {
        &self.address
    }

    /// Returns the key of the encryption
    pub fn get_key(&self) -> &[u8; 32] {
        &self.key
    }

    /// Returns the client id of the server
    pub fn get_id(&self) -> &str {
        &self.id
    }

    /// Returns the username of the monitor
    pub fn get_username(&self) -> &str {
        &self.username
    }

    /// Returns the password of the monitor
    pub fn get_password(&self) -> &str {
        &self.password
    }

    /// Returns the positions of each Drone charging station
    pub fn get_charging_coordenates(&self) -> Vec<Coordenate> {
        self.charging_stations.clone()
    }
}
