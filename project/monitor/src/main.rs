//! The monitor is a program that manages the information received from the drones and the camera system.
//! It has a UI that displays all the information in real time and let's the user create new incidents that will
//! be handled by the drones.

use std::{env::args, path::Path};

use common::error::Error;
use config::Config;

mod camera;
mod channels_tasks;
mod client;
mod config;
mod drone;
mod monitor;
mod right_click_menu;
mod ui_application;

const CLIENT_ARGS: usize = 2;

fn main() -> Result<(), Error> {
    let argv = args().collect::<Vec<String>>();
    if argv.len() != CLIENT_ARGS {
        let app_name = &argv[0];
        return Err(Error::new(format!(
            "Invalid amount of arguments. Usage: {:?} <toml-file>",
            app_name
        )));
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
        return Err(Error::new(format!("Error running client: {:?}", e)));
    }

    Ok(())
}
