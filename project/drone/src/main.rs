//! The drone system is a program that simulates a single drone. It recieves messages from the monitor and
//! reacts to them depending on the situation.

use common::error::Error;
use config::Config;
use std::env::args;
use std::path::Path;

mod client;
mod config;
mod drone;
mod utils;

static CLIENT_ARGS: usize = 2;

fn main() -> Result<(), Error> {
    let argv = args().collect::<Vec<String>>();
    if argv.len() != CLIENT_ARGS {
        let app_name = &argv[0];
        return Err(Error::new(format!("Invalid amount of arguments. Usage: {:?} <config-path>", app_name)));
    }

    let path = Path::new(&argv[1]);

    let config = match Config::from_file(path) {
        Ok(config) => config,
        Err(e) => {
            return Err(Error::new(format!("Error reading config file: {:?}", e)));
        }
    };

    if let Err(e) = client::client_run(config) {
        return Err(Error::new(format!("Error running client: {:?}", e)));
    }

    Ok(())
}
