#![allow(dead_code)]

use std::{fs, path::Path};

/// Represents the configuration of the server
#[derive(Debug, Clone)]
pub struct Config {
    address: String,
    key: [u8; 32],
    username: String,
    password: String,
}

impl Config {
    /// Reads the configuration from a file
    pub fn from_file(path: &Path) -> std::io::Result<Self> {
        let content = fs::read_to_string(path)?;

        let mut config = Config {
            address: String::new(),
            key: [0; 32],
            username: String::new(),
            password: String::new(),
        };

        for line in content.lines() {
            let parts: Vec<&str> = line.split('=').map(|s| s.trim()).collect();
            if parts.len() == 2 {
                match parts[0] {
                    "address" => config.address = parts[1].trim_matches('"').to_string(),
                    "key" => {
                        let key_str = parts[1].trim_matches('"');
                        if key_str.len() != 32 {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "Invalid key length",
                            ));
                        }
                        let mut key = [0; 32];
                        for (i, c) in key_str.chars().enumerate() {
                            key[i] = c as u8;
                        }
                        config.key = key;
                    }
                    "username" => config.username = parts[1].trim_matches('"').to_string(),
                    "password" => config.password = parts[1].trim_matches('"').to_string(),
                    _ => {}
                }
            }
        }

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

    /// Returns the username of the monitor
    pub fn get_username(&self) -> &str {
        &self.username
    }

    /// Returns the password of the monitor
    pub fn get_password(&self) -> &str {
        &self.password
    }
}
