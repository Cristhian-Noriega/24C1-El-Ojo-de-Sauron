use serde_derive::{Deserialize, Serialize};
use std::{fs::File, io::Read, path::Path};

/// Represents the configuration of a drone
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    address: String,
    id: u8,
    username: String,
    password: String,
    key: String,
    x_central_position: f64,
    y_central_position: f64,
    x_anchor_position: f64,
    y_anchor_position: f64,
    velocity: f64,
    active_range: f64,
}

impl Config {
    /// Reads the configuration from a file
    pub fn from_file(path: &Path) -> std::io::Result<Self> {
        let mut file = File::open(path)?;
        let mut contents = String::new();

        file.read_to_string(&mut contents)?;

        let config: Config = serde_json::from_str(&contents)?;

        Ok(config)
    }

    /// Returns the address of the drone
    pub fn get_address(&self) -> &str {
        &self.address
    }

    /// Returns the id of the drone
    pub fn get_id(&self) -> u8 {
        self.id
    }

    /// Returns the username of the drone
    pub fn get_username(&self) -> &str {
        &self.username
    }

    /// Returns the password of the drone
    pub fn get_password(&self) -> &str {
        &self.password
    }

    /// Returns the key of the drone
    pub fn get_key(&self) -> &[u8; 32] {
        self.key.as_bytes().try_into().unwrap()
    }

    /// Returns the x central position of the drone
    pub fn get_x_central_position(&self) -> f64 {
        self.x_central_position
    }

    /// Returns the y central position of the drone
    pub fn get_y_central_position(&self) -> f64 {
        self.y_central_position
    }

    /// Returns the x anchor position of the drone
    pub fn get_x_anchor_position(&self) -> f64 {
        self.x_anchor_position
    }

    /// Returns the y anchor position of the drone
    pub fn get_y_anchor_position(&self) -> f64 {
        self.y_anchor_position
    }

    /// Returns the velocity of the drone
    pub fn get_velocity(&self) -> f64 {
        self.velocity
    }

    /// Returns the active range of the drone
    pub fn get_active_range(&self) -> f64 {
        self.active_range
    }
}
