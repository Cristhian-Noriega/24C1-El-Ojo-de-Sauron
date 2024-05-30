use error::Error;
use std::env;

mod client;
mod config;
mod error;
mod server;
mod task_handler;
mod logfile;

static SERVER_ARGS: usize = 2;

pub fn main() -> Result<(), Error> {
    let argv = env::args().collect::<Vec<String>>();
    if argv.len() != SERVER_ARGS {
        let app_name = &argv[0];
        println!("Usage:\n{:?} <config_file>", app_name);
        return Err(Error::new("Cantidad de argumentos inválido".to_string()));
    }
    println!("\nServer starting with config file: {:?}", argv[1]);

    let config = match config::Config::new(&argv[1]) {
        Some(config) => config,
        None => {
            return Err(Error::new(
                "Error al leer el archivo de configuración".to_string(),
            ))
        }
    };

    let server = server::Server::new(config);
    if let Err(err) = server.server_run() {
        return Err(Error::new(format!(
            "Error al ejecutar el servidor: {:?}",
            err
        )));
    }

    Ok(())
}
