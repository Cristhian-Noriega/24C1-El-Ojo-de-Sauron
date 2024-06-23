use std::{fs, io, path::Path};

/// Represents the configuration of the server
#[derive(Debug, Clone)]
pub struct Config {
    address: String,
    key: [u8; 32],
    log_file: String,
    segs_to_disconnect: u32,
    admin_password: String,
    camera_system_username: String,
    camera_system_password: String,
}

impl Config {
    /// Reads the configuration from a file
    pub fn from_file(path: &Path) -> std::io::Result<Self> {
        let content = fs::read_to_string(path)?;

        let mut config = Config {
            address: String::new(),
            key: [0; 32],
            log_file: String::new(),
            segs_to_disconnect: 0,
            admin_password: String::new(),
            camera_system_username: String::new(),
            camera_system_password: String::new(),
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
                    "segs_to_disconnect" => {
                        config.segs_to_disconnect = parts[1].parse().map_err(|_| {
                            io::Error::new(
                                io::ErrorKind::InvalidData,
                                "Invalid segs_to_disconnect value",
                            )
                        })?
                    }
                    "admin_username" => {
                        config.admin_password = parts[1].trim_matches('"').to_string()
                    }
                    "admin_password" => {
                        config.admin_password = parts[1].trim_matches('"').to_string()
                    }
                    "camera_system_username" => {
                        config.camera_system_username = parts[1].trim_matches('"').to_string()
                    }
                    "camera_system_password" => {
                        config.camera_system_password = parts[1].trim_matches('"').to_string()
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

    // pub fn get_segs_to_disconnect(&self) -> u32 {
    //     self.segs_to_disconnect
    // }

    /// Returns the key of the server
    pub fn get_key(&self) -> &[u8; 32] {
        &self.key
    }

    /// Returns the admin username of the server
    pub fn get_admin_username(&self) -> &str {
        &self.admin_password
    }

    /// Returns the admin password of the server
    pub fn get_admin_password(&self) -> &str {
        &self.admin_password
    }

    /// Returns the camera system username of the server
    pub fn get_camera_system_username(&self) -> &str {
        &self.camera_system_username
    }

    /// Returns the camera system password of the server
    pub fn get_camera_system_password(&self) -> &str {
        &self.camera_system_password
    }
}
