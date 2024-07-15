use common::coordenate::Coordenate;
use std::collections::HashMap;
use std::io;
use std::{fs::File, io::Read, path::Path};

/// Represents the configuration of the camera system
#[derive(Debug, Clone)]
pub struct Config {
    address: String,
    id: String,
    username: String,
    password: String,
    key: String,
    active_range: f64,
    images_folder: String,
    confidence_threshold: f32,
    cameras: Vec<Coordenate>,
}

impl Config {
    /// Reads the configuration from a file
    pub fn from_file(path: &Path) -> std::io::Result<Self> {
        let mut file = File::open(path)?;
        let mut contents = String::new();

        file.read_to_string(&mut contents)?;

        let json = contents.trim().trim_matches(|c| c == '{' || c == '}');

        let mut config_map = HashMap::new();
        let mut cameras = Vec::new();
        let mut inside_cameras = false;
        let mut current_camera = HashMap::new();

        for line in json.lines().map(str::trim).filter(|line| !line.is_empty()) {
            if inside_cameras {
                if line.starts_with('{') {
                    current_camera.clear();
                } else if line.starts_with('}') {
                    if let (Some(x), Some(y)) = (
                        current_camera
                            .get("x_coordinate")
                            .and_then(|v: &String| v.parse::<f64>().ok()),
                        current_camera
                            .get("y_coordinate")
                            .and_then(|v: &String| v.parse::<f64>().ok()),
                    ) {
                        cameras.push(Coordenate {
                            x_coordinate: x,
                            y_coordinate: y,
                        });
                    }
                    current_camera.clear();
                } else {
                    let parts: Vec<&str> = line.splitn(2, ':').collect();

                    if parts.len() != 2 {
                        continue;
                    }

                    let key = parts[0].trim_matches('"').trim();
                    let value = parts[1].trim().trim_matches(|c| c == '"' || c == ',');
                    current_camera.insert(key.to_string(), value.to_string());
                }
            } else {
                let parts: Vec<&str> = line.splitn(2, ':').collect();
                let key = parts[0].trim_matches('"').trim();
                let value = parts[1].trim().trim_matches(|c| c == '"' || c == ',');

                if key == "cameras" {
                    inside_cameras = true;
                    continue;
                }

                config_map.insert(key.to_string(), value.to_string());
            }
        }

        Ok(Config {
            address: config_map
                .remove("address")
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing address"))?,
            id: config_map
                .remove("id")
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing id"))?,
            username: config_map
                .remove("username")
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing username"))?,
            password: config_map
                .remove("password")
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing password"))?,
            key: config_map
                .remove("key")
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing key"))?,
            active_range: config_map
                .remove("active_range")
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing active range"))?
                .parse::<f64>()
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid active_range"))?,
            images_folder: config_map.remove("images_folder").ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "Missing images folder")
            })?,
            confidence_threshold: config_map
                .remove("confidence_threshold")
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing active range"))?
                .parse::<f32>()
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid active_range"))?,
            cameras,
        })
    }

    /// Returns the address of the server
    pub fn get_address(&self) -> &str {
        &self.address
    }

    /// Returns the id of the camera system
    pub fn get_id(&self) -> &str {
        &self.id
    }

    /// Returns the username of the camera system
    pub fn get_username(&self) -> &str {
        &self.username
    }

    /// Returns the password of the camera system
    pub fn get_password(&self) -> &str {
        &self.password
    }

    /// Returns the key of the camera system
    pub fn get_key(&self) -> &[u8; 32] {
        self.key.as_bytes().try_into().unwrap_or(&[0; 32])
    }

    /// Returns the active range of the cameras
    pub fn get_active_range(&self) -> f64 {
        self.active_range
    }

    /// Returns the cameras of the camera system
    pub fn get_cameras(&self) -> Vec<Coordenate> {
        self.cameras.clone()
    }

    /// Returns the path of the root images folder
    pub fn get_images_folder(&self) -> String {
        self.images_folder.clone()
    }

    /// Returns the confidence threshold for the image recognition
    pub fn get_confidence_threshold(&self) -> f32 {
        self.confidence_threshold
    }
}
