// let address = "170.0.0.001".to_owned();
use std::env::args;
use std::io::{BufRead, BufReader, Read};
use std::net::TcpListener;
use std::thread;

static SERVER_ARGS: usize = 2;

fn main() -> Result<(), ()> {
    let argv = args().collect::<Vec<String>>();
    if argv.len() != SERVER_ARGS {
        println!("Cantidad de argumentos inválido");
        let app_name = &argv[0];
        println!("Usage:\n{:?} <puerto>", app_name);
        return Err(());
    }

    let address = "127.0.0.1:".to_owned() + &argv[1];
    server_run(&address).unwrap();
    Ok(())
}

fn server_run(address: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(address)?;

    for mut stream in listener.incoming().flatten() {
        let address = stream.peer_addr().unwrap().to_string();
        println!("Nueva conexión: {:?}", address);

        thread::spawn(move || {
            handle_client(&mut stream, &address).unwrap();
        });
    }

    Ok(())
}

fn handle_client(stream: &mut dyn Read, address: &str) -> std::io::Result<()> {
    let reader = BufReader::new(stream);
    let mut lines = reader.lines();

    while let Some(Ok(line)) = lines.next() {
        println!("Mensaje recibidio desde el cliente: {:?}", address);
        println!("Contenido: {:?}", line);
    }

    Ok(())
}
