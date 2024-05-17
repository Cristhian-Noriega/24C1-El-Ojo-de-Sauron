use std::env;

mod server;

static SERVER_ARGS: usize = 2;

pub fn main() -> Result<(), ()> {
    let argv = env::args().collect::<Vec<String>>();
    if argv.len() != SERVER_ARGS {
        println!("Cantidad de argumentos inválido");
        let app_name = &argv[0];
        println!("Usage:\n{:?} <puerto>", app_name);
        return Err(());
    }

    let address = "127.0.0.1:".to_owned() + &argv[1]; // HARDCODEADO
    if let Err(err) = server_run(&address) {
        println!("Error al ejecutar el servidor: {:?}", err);
        return Err(());
    }

    Ok(())
}