extern crate sauron;
use sauron::build_connect;

fn main() {
    let client_id = b"client1";
    let result = build_connect(client_id);
    for byte in &result {
        print!("{:08b} \n", byte);
    }
}
