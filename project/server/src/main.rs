//! MQTT Server that uses the mqtt library to handle multiple clients concurrently.
//! It recieves messages from the clients and sends them to the corresponding client.

use config::Config;
use error::{ServerError, ServerResult};
use server::Server;
use std::env;
use std::path::Path;

mod client;
mod client_manager;
mod config;
mod error;
mod logfile;
mod server;
mod task_handler;

static SERVER_ARGS: usize = 2;

fn main() -> ServerResult<()> {
    let argv: Vec<String> = env::args().collect();
    if argv.len() != SERVER_ARGS {
        let app_name = &argv[0];
        return Err(ServerError::ArgumentError(format!(
            "Usage: {} <toml-file>",
            app_name
        )));
    }

    let config_path = Path::new(&argv[1]);

    let config = Config::from_file(config_path)?;

    let server = Server::new(config)?;

    server.server_run()
}
