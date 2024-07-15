use std::collections::HashMap;
use std::io;
use std::{fs::File, io::Read, path::Path};

/// Represents the configuration of a drone
#[derive(Debug, Clone)]
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

        let json = contents.trim().trim_matches(|c| c == '{' || c == '}');

        let mut config_map = HashMap::new();

        for line in json.lines().map(str::trim).filter(|line| !line.is_empty()) {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            let key = parts[0].trim_matches('"').trim();
            let value = parts[1].trim().trim_matches(|c| c == '"' || c == ',');

            if parts.len() != 2 {
                continue;
            }

            config_map.insert(key.to_string(), value.to_string());
        }

        Ok(Config {
            address: config_map
                .remove("address")
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing address"))?,
            key: config_map
                .remove("key")
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing key"))?,
            id: config_map
                .remove("id")
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing id"))?
                .parse::<u8>()
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid id"))?,
            username: config_map
                .remove("username")
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing username"))?,
            password: config_map
                .remove("password")
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing password"))?,
            x_central_position: config_map
                .remove("x_central_position")
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData, "Missing x_central_position")
                })?
                .parse::<f64>()
                .map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidData, "Invalid x_central_position")
                })?,
            y_central_position: config_map
                .remove("y_central_position")
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData, "Missing y_central_position")
                })?
                .parse::<f64>()
                .map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidData, "Invalid y_central_position")
                })?,
            x_anchor_position: config_map
                .remove("x_anchor_position")
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData, "Missing x_anchor_position")
                })?
                .parse::<f64>()
                .map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidData, "Invalid x_anchor_position")
                })?,
            y_anchor_position: config_map
                .remove("y_anchor_position")
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData, "Missing y_anchor_position")
                })?
                .parse::<f64>()
                .map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidData, "Invalid y_anchor_position")
                })?,
            velocity: config_map
                .remove("velocity")
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing velocity"))?
                .parse::<f64>()
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid velocity"))?,
            active_range: config_map
                .remove("active_range")
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing active_range"))?
                .parse::<f64>()
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid active_range"))?,
        })
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
        self.key.as_bytes().try_into().unwrap_or(&[0; 32])
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
