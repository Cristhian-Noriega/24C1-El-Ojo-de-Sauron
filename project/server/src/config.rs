use std::{fs, io, path::Path};

#[derive(Debug, Clone)]
pub struct Config {
    address: String,
    log_file: String,
    segs_to_disconnect: u32,
}

impl Config {
    pub fn from_file(path: &Path) -> std::io::Result<Self> {
        let content = fs::read_to_string(path)?;

        let mut config = Config {
            address: String::new(),
            log_file: String::new(),
            segs_to_disconnect: 0,
        };

        for line in content.lines() {
            let parts: Vec<&str> = line.split('=').map(|s| s.trim()).collect();
            if parts.len() == 2 {
                match parts[0] {
                    "address" => config.address = parts[1].trim_matches('"').to_string(),
                    "log_file" => config.log_file = parts[1].trim_matches('"').to_string(),
                    "segs_to_disconnect" => {
                        config.segs_to_disconnect = parts[1].parse().map_err(|_| {
                            io::Error::new(
                                io::ErrorKind::InvalidData,
                                "Invalid segs_to_disconnect value",
                            )
                        })?
                    }
                    _ => {}
                }
            }
        }

        Ok(config)
    }

    pub fn get_address(&self) -> &str {
        &self.address
    }

    pub fn get_log_file(&self) -> &str {
        &self.log_file
    }
}
