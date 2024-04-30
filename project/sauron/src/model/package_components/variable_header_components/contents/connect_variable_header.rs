use std::io::Read;

use crate::{errors::error::Error, model::package_components::fixed_header_components::qos::QoS};

const PROTOCOL_NAME: Vec<u8> = vec![b'M', b'Q', b'T', b'T'];
const PROTOCOL_LEVEL: u8 = 0x04;
const CONNECT_LENGTH: usize = 8;

pub struct ConnectVariableHeader {
    username: bool,
    password: bool,
    will_retain: bool,
    will_qos: QoS,
    will: bool,
    clean_session: bool,
    keep_alive_msb: u8,
    keep_alive_lsb: u8,
}

impl ConnectVariableHeader {
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

    pub fn get_length(self) -> usize {
        CONNECT_LENGTH
    }

    pub fn from_bytes(stream: &mut dyn Read) -> Result<Self, Error> {
        todo!()
    }
}
