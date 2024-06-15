use error::Error;
use std::path::Path;

mod client;
mod config;
mod error;
mod logfile;
mod server;
mod task_handler;

pub fn main() -> Result<(), Error> {
    let path = Path::new("server/Settings.toml");

    let config = config::Config::from_file(&path)?;

    let server = server::Server::new(config);
    if let Err(err) = server.server_run() {
        return Err(Error::new(format!(
            "Error al ejecutar el servidor: {:?}",
            err
        )));
    }

    Ok(())
}
