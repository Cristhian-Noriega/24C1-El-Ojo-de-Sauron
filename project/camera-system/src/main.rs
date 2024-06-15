use std::path::Path;
use config::Config;
use std::env::args;

mod camera;
mod camera_status;
mod camera_system;
mod client;
mod incident;
mod config;

const CLIENT_ARGS: usize = 2;

fn main() {
    let argv = args().collect::<Vec<String>>();
    if argv.len() != CLIENT_ARGS {
        println!("Cantidad de argumentos inválidos");
        let app_name = &argv[0];
        println!("{:?} <config-path>", app_name);

        return;
    }

    let path = Path::new(&argv[1]);

    let config = match Config::from_file(&path) {
        Ok(config) => config,
        Err(e) => {
            println!("Error reading the configuration file: {:?}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = client::client_run(config) {
        println!("Error: {:?}", e);
    }
}

//the client receives a connack packet from the server
