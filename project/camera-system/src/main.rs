use std::path::Path;
use config::Config;

mod camera;
mod camera_status;
mod camera_system;
mod client;
mod incident;
mod config;

fn main() {
    let path = Path::new("monitor/Settings.toml");

    let config = match Config::from_file(&path) {
        Ok(config) => config,
        Err(e) => {
            println!("Error reading the configuration file: {:?}", e);
            std::process::exit(1);
        }
    };

    let address = config.get_address().to_owned() + ":" + config.get_port().to_string().as_str();

    if let Err(e) = client::client_run(&address, config) {
        println!("Error: {:?}", e);
    }
}

//the client receives a connack packet from the server
