use serde::{Deserialize, Serialize};
use std::{fs::File, io::Read, path::Path};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Position {
    pub x_coordinate: f64,
    pub y_coordinate: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    address: String,
    username: String,
    password: String,
    cameras: Vec<Position>,
}

impl Config {
    pub fn from_file(path: &Path) -> std::io::Result<Self> {
        let mut file = File::open(path)?;
        let mut contents = String::new();

        file.read_to_string(&mut contents)?;

        let config = serde_json::from_str(&contents)?;

        Ok(config)
    }

    pub fn get_address(&self) -> &str {
        &self.address
    }

    pub fn get_username(&self) -> &str {
        &self.username
    }

    pub fn get_password(&self) -> &str {
        &self.password
    }

    pub fn get_cameras(&self) -> Vec<Position> {
        self.cameras.clone()
    }
}
