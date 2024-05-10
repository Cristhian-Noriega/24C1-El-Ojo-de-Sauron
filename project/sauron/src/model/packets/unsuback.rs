use std::io::Read;

use crate::{
    errors::error::Error,
    model::{
        fixed_header::FixedHeader, remaining_length::RemainingLength,
    },
};

const RESERVED_FIXED_HEADER_FLAGS: u8 = 0x0B;
const VARIABLE_HEADER_LENGTH: usize = 2;
const PACKET_TYPE: u8 = 0x11;

#[derive(Debug)]
pub struct Unsuback {
    pub packet_identifier: u16,
}

impl Unsuback {
    pub fn new(packet_identifier: u16) -> Self {
        Self {
            packet_identifier,
        }
    }

    pub fn from_bytes(fixed_header: FixedHeader, stream: &mut dyn Read) -> Result<Self, Error> {
        // Fixed Header
        let fixed_header_flags = fixed_header.first_byte() & 0b0000_1111;

        if fixed_header_flags != RESERVED_FIXED_HEADER_FLAGS {
            return Err(Error::new("Invalid flags".to_string()));
        }

        // Variable Header
        let mut variable_header_buffer = vec![0; VARIABLE_HEADER_LENGTH];
        stream.read_exact(&mut variable_header_buffer)?;

        let packet_identifier =
            u16::from_be_bytes([variable_header_buffer[0], variable_header_buffer[1]]);

        Ok(Unsuback::new(packet_identifier))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // Variable Header
        let variable_header_bytes = self.packet_identifier.to_be_bytes().to_vec();

        // Fixed Header
        let mut fixed_header_bytes = vec![PACKET_TYPE << 4 | RESERVED_FIXED_HEADER_FLAGS];

        let remaining_length_value = variable_header_bytes.len() as u32;
        let remaining_length_bytes = RemainingLength::new(remaining_length_value).to_bytes();
        fixed_header_bytes.extend(remaining_length_bytes);

        let mut packet_bytes = vec![];

        packet_bytes.extend(fixed_header_bytes);
        packet_bytes.extend(variable_header_bytes);

        packet_bytes
    }
}
