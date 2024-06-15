use std::{
    path::Path,
    fs::File,
    io::Read,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Position {
    pub x_coordinate: f64,
    pub y_coordinate: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    port: u16,
    address: String,
    cameras: Vec<Position>,
}

impl Config {
    pub fn from_file(path: &Path) -> std::io::Result<Self> {
        let mut file = File::open(path)?;
        let mut contents = String::new();

        file.read_to_string(&mut contents)?;

        let config: Config = serde_json::from_str(&contents)?;

        Ok(config)
    }

    pub fn get_port(&self) -> u16 {
        self.port
    }

    pub fn get_address(&self) -> &str {
        &self.address
    }

    pub fn get_cameras(&self) -> Vec<Position> {
        self.cameras.clone()
    }
}
