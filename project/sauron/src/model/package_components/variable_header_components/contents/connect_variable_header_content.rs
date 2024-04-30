use std::io::Read;

use crate::{errors::error::Error, model::package_components::fixed_header_components::qos::QoS};

const CONNECT_LENGTH: usize = 8;

const PROTOCOL_NAME: [u8; 4] = [b'M', b'Q', b'T', b'T'];
const PROTOCOL_LEVEL: u8 = 0x04;

pub struct ConnectVariableHeaderContent {
    username: bool,
    password: bool,
    will_retain: bool,
    will_qos: QoS,
    will: bool,
    clean_session: bool,
    keep_alive_msb: u8,
    keep_alive_lsb: u8,
}

impl ConnectVariableHeaderContent {
    pub fn new(
        username: bool,
        password: bool,
        will_retain: bool,
        will_qos: QoS,
        will: bool,
        clean_session: bool,
        keep_alive_msb: u8,
        keep_alive_lsb: u8,
    ) -> Self {
        Self {
            username,
            password,
            will_retain,
            will_qos,
            will,
            clean_session,
            keep_alive_msb,
            keep_alive_lsb,
        }
    }

    pub fn into_bytes(self) -> Vec<u8> {
        let flags_byte = (self.username as u8) << 7
            | (self.password as u8) << 6
            | (self.will_retain as u8) << 5
            | (self.will_qos.into_byte() << 3)
            | (self.will as u8) << 2
            | (self.clean_session as u8) << 1;

        let mut conect_bytes = vec![];

        conect_bytes.extend(PROTOCOL_NAME);
        conect_bytes.push(PROTOCOL_LEVEL);
        conect_bytes.push(flags_byte);
        conect_bytes.push(self.keep_alive_msb);
        conect_bytes.push(self.keep_alive_lsb);

        conect_bytes
    }

    pub fn get_length(&self) -> usize {
        CONNECT_LENGTH
    }

    pub fn from_bytes(stream: &mut dyn Read) -> Result<Self, Error> {
        let mut buffer = [0; CONNECT_LENGTH];
        stream.read_exact(&mut buffer)?;

        // debería seguir MQTT
        for letter in PROTOCOL_NAME.iter() {
            if *letter != buffer[0] {
                return Err(Error::new("Invalid protocol name".to_string()));
            }
        }

        if buffer[4] != PROTOCOL_LEVEL {
            return Err(Error::new("Invalid protocol level".to_string()));
        }

        let flags_byte = buffer[5];
        let username = (flags_byte & 0b1000_0000) >> 7 == 1;
        let password = (flags_byte & 0b0100_0000) >> 6 == 1;
        let will_retain = (flags_byte & 0b0010_0000) >> 5 == 1;
        let will_qos = QoS::from_byte((flags_byte & 0b0001_1000) >> 3)?;
        let will = (flags_byte & 0b0000_0100) >> 2 == 1;
        let clean_session = (flags_byte & 0b0000_0010) >> 1 == 1;

        let keep_alive_msb = buffer[6];
        let keep_alive_lsb = buffer[7];

        Ok(Self {
            username,
            password,
            will_retain,
            will_qos,
            will,
            clean_session,
            keep_alive_msb,
            keep_alive_lsb,
        })
    }
}
