use serde_derive::{Deserialize, Serialize};
use std::{fs::File, io::Read, path::Path};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    address: String,
    id: u8,
    x_position: f64,
    y_position: f64,
    x_central_position: f64,
    y_central_position: f64,
    x_anchor_position: f64,
    y_anchor_position: f64,
    velocity: f64,
    active_range: f64,
}

impl Config {
    pub fn from_file(path: &Path) -> std::io::Result<Self> {
        let mut file = File::open(path)?;
        let mut contents = String::new();

        file.read_to_string(&mut contents)?;

        let config: Config = serde_json::from_str(&contents)?;

        Ok(config)
    }

    pub fn get_address(&self) -> &str {
        &self.address
    }

    pub fn get_id(&self) -> u8 {
        self.id
    }

    pub fn get_x_central_position(&self) -> f64 {
        self.x_central_position
    }

    pub fn get_y_central_position(&self) -> f64 {
        self.y_central_position
    }

    pub fn get_x_anchor_position(&self) -> f64 {
        self.x_anchor_position
    }

    pub fn get_y_anchor_position(&self) -> f64 {
        self.y_anchor_position
    }

    pub fn get_velocity(&self) -> f64 {
        self.velocity
    }

    pub fn get_active_range(&self) -> f64 {
        self.active_range
    }
}
