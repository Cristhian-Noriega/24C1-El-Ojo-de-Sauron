use std::io::Read;

use crate::{errors::error::Error, model::package_components::fixed_header_components::qos::QoS};

const CONNACK_LENGTH: usize = 8;

const PROTOCOL_NAME: [u8; 4] = [b'M', b'Q', b'T', b'T'];
const PROTOCOL_LEVEL: u8 = 0x04;

pub struct VariableHeaderContentConnack {
    username: bool,
    password: bool,
    will_retain: bool,
    will_qos: QoS,
    will: bool,
    clean_session: bool,
    keep_alive: u16,
}

impl VariableHeaderContentConnack {
    pub fn new(
        clean_session: bool,
        will: bool,
        will_qos: QoS,
        will_retain: bool,
        username: bool,
        password: bool,
        keep_alive: u16,
    ) -> Self {
        Self {
            clean_session,
            will,
            will_qos,
            will_retain,
            username,
            password,
            keep_alive,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let flags_byte = (self.clean_session as u8) << 1
            | (self.will as u8) << 2
            | (self.will_qos.to_byte() << 3)
            | (self.will_retain as u8) << 5
            | (self.password as u8) << 6
            | (self.username as u8) << 7;

        let mut connack_bytes = vec![];

        connack_bytes.extend(PROTOCOL_NAME);
        connack_bytes.push(PROTOCOL_LEVEL);
        connack_bytes.push(flags_byte);
        connack_bytes.extend(&self.keep_alive.to_be_bytes());

        connack_bytes
    }

    pub fn get_length(&self) -> usize {
        CONNACK_LENGTH
    }

    pub fn from_bytes(stream: &mut dyn Read) -> Result<Self, Error> {
        let mut buffer = [0; CONNACK_LENGTH];
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

        if (flags_byte & 0b0000_0001) != 0 {
            return Err(Error::new("Invalid connack flags".to_string()));
        }

        let clean_session = (flags_byte & 0b0000_0010) >> 1 == 1;
        let will = (flags_byte & 0b0000_0100) >> 2 == 1;
        let will_qos = QoS::from_byte((flags_byte & 0b0001_1000) >> 3)?;
        let will_retain = (flags_byte & 0b0010_0000) >> 5 == 1;

        if !will && will_qos != QoS::AtMost {
            return Err(Error::new("Invalid will qos".to_string()));
        }

        if !will && will_retain {
            return Err(Error::new("Invalid will retain flag".to_string()));
        }

        let username = (flags_byte & 0b1000_0000) >> 7 == 1;
        let password = (flags_byte & 0b0100_0000) >> 6 == 1;

        if !username && password {
            return Err(Error::new("Invalid password flag".to_string()));
        }

        let keep_alive = u16::from_be_bytes([buffer[6], buffer[7]]);

        Ok(Self {
            username,
            password,
            will_retain,
            will_qos,
            will,
            clean_session,
            keep_alive,
        })
    }

    pub fn has_will(&self) -> bool {
        self.will
    }

    pub fn has_username(&self) -> bool {
        self.username
    }

    pub fn has_password(&self) -> bool {
        self.password
    }
}
