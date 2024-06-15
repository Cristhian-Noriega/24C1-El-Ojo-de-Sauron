use std::{
    path::Path,
    fs,
    io
};

#[derive(Debug, Clone)]
pub struct Config {
    port: u16,
    address: String,
}

impl Config {
    pub fn from_file(path: &Path) -> std::io::Result<Self> {
        let content = fs::read_to_string(path)?;

        let mut config = Config {
            port: 0,
            address: String::new(),
        };

        for line in content.lines() {
            let parts: Vec<&str> = line.split('=').map(|s| s.trim()).collect();
            if parts.len() == 2 {
                match parts[0] {
                    "port" => config.port = parts[1].parse().map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid port value"))?,
                    "address" => config.address = parts[1].trim_matches('"').to_string(),
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
}
