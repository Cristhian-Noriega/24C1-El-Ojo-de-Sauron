use std::{env::args, path::Path};

use config::Config;

mod camera;
mod channels_tasks;
mod client;
mod config;
mod drone;
mod monitor;
mod ui_application;

const CLIENT_ARGS: usize = 2;

fn main() {
    let argv = args().collect::<Vec<String>>();
    if argv.len() != CLIENT_ARGS {
        println!("Cantidad de argumentos inválidos");
        let app_name = &argv[0];
        println!("{:?} <toml-file>", app_name);

        return;
    }

    let path = Path::new(&argv[1]);

    let config = match Config::from_file(path) {
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
