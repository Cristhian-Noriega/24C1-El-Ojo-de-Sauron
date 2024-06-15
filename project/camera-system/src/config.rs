use std::{
    path::Path,
    fs,
    io
};

#[derive(Debug, Clone)]
pub struct Config {
    port: u16,
    address: String,
    number_of_cameras: usize,
}

impl Config {
    pub fn from_file(path: &Path) -> std::io::Result<Self> {
        let content = fs::read_to_string(path)?;

        let mut config = Config {
            port: 0,
            address: String::new(),
            number_of_cameras: 0,
        };

        for line in content.lines() {
            let parts: Vec<&str> = line.split('=').map(|s| s.trim()).collect();
            if parts.len() == 2 {
                match parts[0] {
                    "port" => config.port = parts[1].parse().map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid port value"))?,
                    "address" => config.address = parts[1].trim_matches('"').to_string(),
                    "number_of_cameras" => config.number_of_cameras = parts[1].parse().map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid segs_to_disconnect value"))?,
                    _ => {}
                }
            }
        }

        Ok(config)
    }

    pub fn get_port(&self) -> u16 {
        self.port
    }

    pub fn get_address(&self) -> &str {
        &self.address
    }

    pub fn get_number_of_cameras(&self) -> usize {
        self.number_of_cameras
    }
}
