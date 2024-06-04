// #![allow(dead_code)]

use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Read},
};

#[derive(Debug, Clone)]
pub struct Config {
    port: u16,
    address: String,
    log_file: String,
}

impl Config {
    pub fn new(path_file: &str) -> Option<Config> {
        let config_file = File::open(path_file).ok()?;
        Config::from_file(config_file)
    }

    pub fn from_file<R: Read>(config_file: R) -> Option<Config> {
        let buf_reader = BufReader::new(config_file);
        let config_map = Self::parse_file_to_map(buf_reader)?;

        let port = config_map.get("port")?.parse().ok()?;
        let address = config_map.get("address")?.clone();
        let log_file = config_map.get("log_file")?.clone();

        Some(Config {
            port,
            address,
            log_file,
        })
    }

    fn parse_file_to_map<R: BufRead>(reader: R) -> Option<HashMap<String, String>> {
        let mut config_map = HashMap::new();
        for line in reader.lines() {
            let line = line.ok()?;
            let mut parts = line.splitn(2, '=');
            if let (Some(k), Some(value)) = (parts.next(), parts.next()) {
                config_map.insert(k.trim().to_string(), value.trim().to_string());
            }
        }
        Some(config_map)
    }

    pub fn get_port(&self) -> u16 {
        self.port
    }

    pub fn get_address(&self) -> &str {
        &self.address
    }

    pub fn get_log_file(&self) -> &str {
        &self.log_file
    }

}
