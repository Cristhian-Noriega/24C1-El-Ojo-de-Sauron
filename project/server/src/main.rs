use error::Error;
use std::path::Path;
use std::env;

mod client;
mod config;
mod error;
mod logfile;
mod server;
mod task_handler;

static SERVER_ARGS: usize = 2;

pub fn main() -> Result<(), Error> {
    let argv = env::args().collect::<Vec<String>>();
    if argv.len() != SERVER_ARGS {
        let app_name = &argv[0];
        println!("Usage:\n{:?} <toml-file>", app_name);
        return Err(Error::new("Cantidad de argumentos inválido".to_string()));
    }

    let path = Path::new(&argv[1]);

    let config = config::Config::from_file(path)?;

    let server = server::Server::new(config);
    if let Err(err) = server.server_run() {
        return Err(Error::new(format!(
            "Error al ejecutar el servidor: {:?}",
            err
        )));
    }

    Ok(())
}
