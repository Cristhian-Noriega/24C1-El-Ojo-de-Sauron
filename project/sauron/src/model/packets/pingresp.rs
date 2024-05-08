use crate::{errors::error::Error, model::fixed_header::FixedHeader};

const PACKET_TYPE: u8 = 0x13;
const RESERVED_FIXED_HEADER_FLAGS: u8 = 0x00;

#[derive(Debug, Default)]
pub struct Pingresp;

impl Pingresp {
    pub fn new() -> Self {
        Self
    }

    pub fn from_bytes(fixed_header: FixedHeader) -> Result<Self, Error> {
        // Fixed Header
        let fixed_header_flags = fixed_header.first_byte() & 0b0000_1111;

        if fixed_header_flags != RESERVED_FIXED_HEADER_FLAGS {
            return Err(Error::new("Invalid reserved header flags".to_string()));
        }

        Ok(Pingresp::new())
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // Fixed Header
        let remaining_length: u8 = 0x00;

        let packet_bytes = vec![
            PACKET_TYPE << 4 | RESERVED_FIXED_HEADER_FLAGS,
            remaining_length,
        ];

        packet_bytes
    }
}
