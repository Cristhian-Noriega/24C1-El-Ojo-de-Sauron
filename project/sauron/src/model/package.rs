use crate::model::package_components::{
    fixed_header::FixedHeader,
    fixed_header_components::{
        control_packet_type::ControlPacketType::Connect, flags::Flags, qos::QoS,
    },
    variable_header::VariableHeader,
};
use std::io::{Error, ErrorKind};

// siento que esto no debería estar aquí
const PACKET_IDENTIFIER_MSB: u8 = 0x00;
const PACKET_IDENTIFIER_LSB: u8 = 0x04;
const PROTOCOL_LEVEL: u8 = 0x04;
const CLEAN_SESSION: u8 = 0x02;
const KEEP_ALIVE_MSB: u8 = 10;
const KEEP_ALIVE_LSB: u8 = 10;

pub struct Package {
    fixed_header: FixedHeader,
    variable_header: VariableHeader,
    payload: Vec<u8>,
}

impl Package {
    pub fn build_connect(client_id: &[u8]) -> Result<Self, std::io::Error> {
        let fixed_header = FixedHeader::new(
            Connect,
            Flags::new(false, false, QoS::AtMostOnce),
            (10 + client_id.len()) as u8,
        );

        let variable_header = VariableHeader::new(
            PACKET_IDENTIFIER_MSB,
            PACKET_IDENTIFIER_LSB,
            vec![
                'M' as u8,
                'Q' as u8,
                'T' as u8,
                'T' as u8,
                PROTOCOL_LEVEL,
                CLEAN_SESSION,
                KEEP_ALIVE_MSB,
                KEEP_ALIVE_LSB,
            ],
        );

        let mut payload: Vec<u8> = vec![];
        payload.extend_from_slice(client_id);

        Ok(Package {
            fixed_header,
            variable_header,
            payload,
        })
    }

    pub fn convert(self) -> Vec<u8> {
        let mut message: Vec<u8> = vec![];
        let fixed_header_binary = self.fixed_header.convert_to_binary();
        //let variable_header_binary = self.variable_header.convert_to_binary();

        message.extend(fixed_header_binary);
        // message.extend(variable_header_binary);
        message.extend(self.payload);

        //let remaining_length = variable_header_binary.len() + payload.len();
        //message[1] = remaining_length as u8;

        return message
    }
}
