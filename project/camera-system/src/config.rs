use serde_derive::{Deserialize, Serialize};
use std::{fs::File, io::Read, path::Path};

/// Represents a position in 2D space
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Position {
    pub x_coordinate: f64,
    pub y_coordinate: f64,
}

/// Represents the configuration of the camera system
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    address: String,
    username: String,
    password: String,
    key: [u8; 32],
    cameras: Vec<Position>,
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

    /// Returns the username of the camera system
    pub fn get_username(&self) -> &str {
        &self.username
    }

    /// Returns the password of the camera system
    pub fn get_password(&self) -> &str {
        &self.password
    }

    /// Returns the key of the camera system
    pub fn get_key(&self) -> &[u8; 32] {
        &self.key
    }

    /// Returns the cameras of the camera system
    pub fn get_cameras(&self) -> Vec<Position> {
        self.cameras.clone()
    }
}
