use common::coordenate::Coordenate;
use std::collections::HashMap;
use std::{fs::File, io::Read, path::Path};
use std::io;

/// Represents the configuration of the server
#[derive(Debug, Clone)]
pub struct Config {
    address: String,
    key: String,
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

        let json = contents.trim().trim_matches(|c| c == '{' || c == '}');

        let mut config_map = HashMap::new();
        let mut charging_stations = Vec::new();
        let mut inside_charging_stations = false;
        let mut current_station = HashMap::new();

        for line in json.lines().map(str::trim).filter(|line| !line.is_empty()) {
            if inside_charging_stations {
                if line.starts_with('{') {
                    current_station.clear();
                } else if line.starts_with('}') {
                    if let (Some(x), Some(y)) = (
                        current_station.get("x_coordinate").and_then(|v: &String| v.parse::<f64>().ok()),
                        current_station.get("y_coordinate").and_then(|v: &String| v.parse::<f64>().ok()),
                    ) {
                        charging_stations.push(Coordenate { x_coordinate: x, y_coordinate: y });
                    }
                    current_station.clear();
                } else {
                    let parts: Vec<&str> = line.splitn(2, ':').collect();

                    if parts.len() != 2 {
                        continue;
                    }

                    let key = parts[0].trim_matches('"').trim();
                    let value = parts[1].trim().trim_matches(|c| c == '"' || c == ',');
                    current_station.insert(key.to_string(), value.to_string());
                }
            } else {
                let parts: Vec<&str> = line.splitn(2, ':').collect();
                let key = parts[0].trim_matches('"').trim();
                let value = parts[1].trim().trim_matches(|c| c == '"' || c == ',');

                if key == "charging_stations" {
                    inside_charging_stations = true;
                    continue;
                }

                config_map.insert(key.to_string(), value.to_string());
            }            
        }

        Ok(Config {
            address: config_map.remove("address").ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing address"))?,
            key: config_map.remove("key").ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing key"))?,
            id: config_map.remove("id").ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing id"))?,
            username: config_map.remove("username").ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing username"))?,
            password: config_map.remove("password").ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing password"))?,
            charging_stations,
        })
    }

    /// Returns the address of the server
    pub fn get_address(&self) -> &str {
        &self.address
    }

    /// Returns the key of the encryption
    pub fn get_key(&self) -> &[u8; 32] {
        self.key.as_bytes().try_into().unwrap_or(&[0; 32])
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
