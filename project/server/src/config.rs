

//Configuración: debe incluir todos los parámetros necesarios para la ejecución del servidor, como el puerto, direccion IP, etc.

//(no esta permitido definir estos valores mediante constantes en el código)
// Config contains the configuration for the server to run it

use std::{fs::File, io::{BufRead, BufReader, Read}};

pub struct Config {
port: u16,
address: String,
log_file: String,
segs_to_disconnect: u32,
}

impl Config {
    pub fn new(path_file: &str) -> Option<Config> {
        let config_file = File::open(&path_file).ok()?;
        Config::from_file(config_file)
    }

    pub fn from_file<R: Read>(config_file: R) -> Option<Config> {
        let mut buf_reader = BufReader::new(config_file);
        let mut port = Self::get_value_from_file(&mut buf_reader, "port")?;
        let mut address = Self::get_value_from_file(&mut buf_reader, "address")?;
        let mut log_file = Self::get_value_from_file(&mut buf_reader, "log_file")?;
        let mut segs_to_disconnect = Self::get_value_from_file(&mut buf_reader, "segs_to_disconnect")?;
        
        let segs_to_disconnect = segs_to_disconnect.parse().ok()?;
        let port = port.parse().ok()?;

        Some(Config {
            port,
            address,
            log_file,
            segs_to_disconnect,
        })
    }

    fn get_value_from_file<R: BufRead>(reader: &mut R, key: &str) -> Option<String> {
        for line in reader.lines() {
            let line = line.ok()?;
            let mut parts = line.splitn(2, '=');
            if let (Some(k), Some(value)) = (parts.next(), parts.next()) {
                if k.trim() == key {
                    return Some(value.trim().to_string());
                }
            }
        }
        None
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

    pub fn get_segs_to_disconnect(&self) -> u32 {
        self.segs_to_disconnect
    }
}

  

// Tests

  

// #[cfg(test)]

// mod tests {

// use super::*;

  

// #[test]

// fn test_valid_config_file() {

// let config_data = r#"

// port=8080

// address=127.0.0.1

// log_file=/var/log/server.log

// segs_to_disconnect=5

// "#;

// let mut cursor = Cursor::new(config_data);

// let config = Config::from_file(&mut cursor).unwrap();

  

// assert_eq!(config.port, 8080);

// assert_eq!(config.address, "127.0.0.1");

// assert_eq!(config.log_file, "/var/log/server.log");

// assert_eq!(config.segs_to_disconnect, 5);

// }

// }