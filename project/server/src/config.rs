use std::{fs, io, path::Path};

/// Represents the configuration of the server
#[derive(Debug, Clone)]
pub struct Config {
    address: String,
    key: [u8; 32],
    log_file: String,
    login_file: String,
    segs_to_disconnect: u32,
    initialize_with_backup: bool,
    backup_file: String,
    segs_to_backup: u32,
}

impl Config {
    /// Reads the configuration from a file
    pub fn from_file(path: &Path) -> std::io::Result<Self> {
        let content = fs::read_to_string(path)?;

        let mut config = Config {
            address: String::new(),
            key: [0; 32],
            log_file: String::new(),
            login_file: String::new(),
            segs_to_disconnect: 0,
            initialize_with_backup: false,
            backup_file: String::new(),
            segs_to_backup: 0,
        };

        for line in content.lines() {
            let parts: Vec<&str> = line.split('=').map(|s| s.trim()).collect();
            if parts.len() == 2 {
                match parts[0] {
                    "address" => config.address = parts[1].trim_matches('"').to_string(),
                    "key" => {
                        let key_str = parts[1].trim_matches('"');
                        if key_str.len() != 32 {
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                "Invalid key length",
                            ));
                        }
                        let mut key = [0; 32];
                        for (i, c) in key_str.chars().enumerate() {
                            key[i] = c as u8;
                        }
                        config.key = key;
                    }
                    "log_file" => config.log_file = parts[1].trim_matches('"').to_string(),
                    "login_file" => config.login_file = parts[1].trim_matches('"').to_string(),
                    "segs_to_disconnect" => {
                        config.segs_to_disconnect = parts[1].parse().map_err(|_| {
                            io::Error::new(
                                io::ErrorKind::InvalidData,
                                "Invalid segs_to_disconnect value",
                            )
                        })?
                    }
                    "initialize_with_backup" => {
                        config.initialize_with_backup = match parts[1].to_lowercase().as_str() {
                            "true" => true,
                            _ => false,
                        }
                    }
                    "backup_file" => config.backup_file = parts[1].trim_matches('"').to_string(),
                    "segs_to_backup" => {
                        config.segs_to_backup = parts[1].parse().map_err(|_| {
                            io::Error::new(
                                io::ErrorKind::InvalidData,
                                "Invalid segs_to_backup value",
                            )
                        })?
                    }
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

    /// Returns the log file of the server
    pub fn get_log_file(&self) -> &str {
        &self.log_file
    }

    /// Returns the login file of the server
    pub fn get_login_file(&self) -> &str {
        &self.login_file
    }

    /// Returns the key of the encryption
    pub fn get_key(&self) -> &[u8; 32] {
        &self.key
    }

    pub fn get_backup_file(&self) -> Option<String> {
        if self.backup_file.is_empty() {
            None
        } else {
            Some(self.backup_file.clone())
        }
    }

    pub fn get_initialize_with_backup(&self) -> bool {
        self.initialize_with_backup
    }

    pub fn get_segs_to_backup(&self) -> u32 {
        self.segs_to_backup
    }
}
