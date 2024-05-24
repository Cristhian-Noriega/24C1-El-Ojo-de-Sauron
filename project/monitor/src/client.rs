use std::env::args;

static CLIENT_ARGS: usize = 3;

pub struct Client {
    pub response: String,
    pub address: String,
}

impl Client {
    pub fn new() -> Self {
        let argv = args().collect::<Vec<String>>();
        if argv.len() != CLIENT_ARGS {
            let app_name = &argv[0];
            println!("{:?} <host> <puerto>", app_name);
        }

        let address = argv[1].clone() + ":" + &argv[2];

        Self {
            response: "no response".to_owned(),
            address: address,
        }
    }

    pub fn send_connect(&mut self) {
        println!("Connecting to {}", self.address);
        self.response = "Connected!".to_owned();
    }
}