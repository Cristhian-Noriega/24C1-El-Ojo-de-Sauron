use std::io::Read;

use crate::{
    errors::error::Error,
    model::fixed_header::FixedHeader,
    model::return_codes::suback_return_code::SubackReturnCode,
};

const RESERVED_FIXED_HEADER_FLAGS: u8 = 0x02;
const VARIABLE_HEADER_LENGTH: usize = 2;
const PACKET_TYPE: u8 = 0x09;

#[derive(Debug)]
pub struct Suback {
    pub packet_identifier: u16,
    pub suback_return_codes: Vec<SubackReturnCode>,
}

impl Suback {
    pub fn new(packet_identifier: u16, suback_return_codes: Vec<SubackReturnCode>) -> Self {
        Self {
            packet_identifier,
            suback_return_codes,
        }
    }

    pub fn from_bytes(fixed_header: FixedHeader, stream: &mut dyn Read) -> Result<Self, Error> {
        // Fixed Header
        let fixed_header_flags = fixed_header.first_byte() & 0b0000_1111;

        if fixed_header_flags != RESERVED_FIXED_HEADER_FLAGS {
            return Err(Error::new("Invalid flags".to_string()));
        }

        let remaining_length = fixed_header.remaining_length() as usize;

        if remaining_length < VARIABLE_HEADER_LENGTH {
            return Err(Error::new("Invalid remaining length".to_string()));
        }

        // Variable Header
        let mut variable_header_buffer = vec![0; VARIABLE_HEADER_LENGTH];
        stream.read_exact(&mut variable_header_buffer)?;

        let packet_identifier =
            u16::from_be_bytes([variable_header_buffer[0], variable_header_buffer[1]]);

        let mut return_codes = vec![];

        // Payload
        let mut payload_buffer = vec![0; remaining_length - VARIABLE_HEADER_LENGTH];
        stream.read_exact(&mut payload_buffer)?;

        for &return_code_byte in payload_buffer.iter() {
            let return_code = SubackReturnCode::from_byte(return_code_byte)?;
            return_codes.push(return_code);
        }

        Ok(Suback::new(packet_identifier, return_codes))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // Variable Header
        let mut variable_header_bytes = self.packet_identifier.to_be_bytes().to_vec();

        // Payload
        for return_code in &self.suback_return_codes {
            variable_header_bytes.push(return_code.to_byte());
        }

        // Fixed Header
        let remaining_length = variable_header_bytes.len();
        let fixed_header_bytes = vec![
            PACKET_TYPE << 4 | RESERVED_FIXED_HEADER_FLAGS,
            remaining_length as u8,
        ];

        let mut packet_bytes = vec![];

        packet_bytes.extend(fixed_header_bytes);
        packet_bytes.extend(variable_header_bytes);

        packet_bytes
    }
}
