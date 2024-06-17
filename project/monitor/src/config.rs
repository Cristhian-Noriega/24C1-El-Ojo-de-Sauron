use std::{fs, path::Path};

#[derive(Debug, Clone)]
pub struct Config {
    address: String,
}

impl Config {
    pub fn from_file(path: &Path) -> std::io::Result<Self> {
        let content = fs::read_to_string(path)?;

        let mut config = Config {
            address: String::new(),
        };

        for line in content.lines() {
            let parts: Vec<&str> = line.split('=').map(|s| s.trim()).collect();
            if parts.len() == 2 && parts[0] == "address" {
                config.address = parts[1].trim_matches('"').to_string()
            }
        }

        Ok(config)
    }

    pub fn get_address(&self) -> &str {
        &self.address
    }
}
