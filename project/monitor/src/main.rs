use std::env::args;

mod camera;
mod channels_tasks;
mod client;
mod drone;
mod monitor;
mod ui_application;
mod config;

const CLIENT_ARGS: usize = 3;

fn main() {
    let argv = args().collect::<Vec<String>>();
    if argv.len() != CLIENT_ARGS {
        println!("Cantidad de argumentos inválidos");
        let app_name = &argv[0];
        println!("{:?} <host> <puerto>", app_name);

        return;
    }

    let address = argv[1].clone() + ":" + &argv[2];

    if let Err(e) = client::client_run(address) {
        println!("Error: {:?}", e);
    }
}
