use error::Error;
use std::env;

mod client;
mod config;
mod error;
mod server;
mod task_handler;

static SERVER_ARGS: usize = 2;

pub fn main() -> Result<(), Error> {
    let argv = env::args().collect::<Vec<String>>();
    if argv.len() != SERVER_ARGS {
        let app_name = &argv[0];
        println!("Usage:\n{:?} <puerto>", app_name);
        return Err(Error::new("Cantidad de argumentos inválido".to_string()));
    }

    let address = "127.0.0.1:".to_owned() + &argv[1]; // HARDCODEADO
    if let Err(err) = server::Server::new().server_run(&address) {
        return Err(Error::new(format!(
            "Error al ejecutar el servidor: {:?}",
            err
        )));
    }

    Ok(())
}
