use std::env;
use error::Error;

mod client;
mod config;
mod server;
mod task_handler;
mod error;

static SERVER_ARGS: usize = 2;

pub fn main() -> Result<(), Error>{
    let argv = env::args().collect::<Vec<String>>();
    if argv.len() != SERVER_ARGS {
        println!("Cantidad de argumentos inválido");
        let app_name = &argv[0];
        println!("Usage:\n{:?} <puerto>", app_name);
        return Err(Error::new("Cantidad de argumentos inválido".to_owned()));
    }

    let address = "127.0.0.1:".to_owned() + &argv[1]; // HARDCODEADO
    if let Err(err) = server::Server::new().server_run(&address) {
        println!("Error al ejecutar el servidor: {:?}", err);
        return Err(Error::new("Error al ejecutar el servidor".to_owned()));
    }

    Ok(())
}
