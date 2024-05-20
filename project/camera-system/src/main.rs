use std::env::args;
use std::io::stdin;
use std::io::Write;
use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;

use sauron::model::components::encoded_string::EncodedString;
pub use sauron::model::{
    packet::Packet,
    packets::{
        connect::Connect, disconnect::Disconnect, pingresp::Pingresp, puback::Puback,
        publish::Publish, subscribe::Subscribe, unsubscribe::Unsubscribe,
    },
};


static CLIENT_ARGS: usize = 3;

fn main() -> Result<(), ()> {
    let argv = args().collect::<Vec<String>>();
    if argv.len() != CLIENT_ARGS {
        println!("Cantidad de argumentos inválido");
        let app_name = &argv[0];
        println!("{:?} <host> <puerto>", app_name);
        return Err(());
    }

    let address = argv[1].clone() + ":" + &argv[2];
    println!("Conectándome a {:?}", address);

    match client_run(&address, &mut stdin()) {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("Error: {:?}", e);
            Err(())
        }
    }
}

fn client_run(address: &str, from_server_stream: &mut dyn Read) -> std::io::Result<()> {
    let mut to_server_stream = TcpStream::connect(address)?;
    let client_id_bytes :Vec<u8> = vec![b'i', b'd'];
    let client_id = EncodedString::new(client_id_bytes);
    let will = None; 
    let login = None;
    let connect_package = Connect::new(false, 0, client_id, will, login);
    let _ = to_server_stream.write(connect_package.to_bytes().as_slice());
    
    // let reader = BufReader::new(from_server_stream);


    Ok(())
}
