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
    if let Err(err) = server_run(&address) {
        println!("Error al ejecutar el servidor: {:?}", err);
        return Err(());
    }

    Ok(())
}

fn server_run(address: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(address)?;

    for stream_result in listener.incoming() {
        match stream_result {
            Ok(mut stream) => {
                let address = stream.peer_addr()?.to_string();
                println!("Nueva conexión: {:?}", address);

                thread::spawn(move || {
                    if let Err(err) = handle_client(&mut stream, &address) {
                        println!("Error al manejar el cliente: {:?}", err);
                    }
                });
            }
            Err(err) => {
                println!("Error al aceptar la conexión: {:?}", err);
            }
        }
    }

    Ok(())
}

#[allow(dead_code)]
fn wait_connection(listener: TcpListener) {
    if listener.set_nonblocking(true).is_err() {
        println!("Error al setear el socket como no bloqueante");
    }
    todo!();
}

#[allow(dead_code)]
fn handle_client(stream: &mut dyn Read, address: &str) -> std::io::Result<()> {
    let reader = BufReader::new(stream);
    let mut lines = reader.lines();

    while let Some(Ok(line)) = lines.next() {
        println!("Mensaje recibidio desde el cliente: {:?}", address);
        println!("Contenido: {:?}", line);
    }

    Ok(())
}
