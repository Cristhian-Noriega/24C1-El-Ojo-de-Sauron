use std::env::args;

static CLIENT_ARGS: usize = 3;

pub struct Client {
    pub connection_status: String,
    pub response_text: String,
    pub response_bytes: String,
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
            connection_status: "offline".to_owned(),
            response_text: "no response".to_owned(),
            response_bytes: "no response".to_owned(),
            address: address,
        }
    }

    pub fn send_connect(&mut self) {
        println!("Connecting to {}", self.address);
        self.connection_status = "connected".to_owned();
        self.response_text = "CONNACK".to_owned();
        self.response_bytes = "CONNACK".to_owned();
    }
}