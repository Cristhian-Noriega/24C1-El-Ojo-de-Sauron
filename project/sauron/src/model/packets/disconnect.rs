use crate::{
    errors::error::Error,
    model::{fixed_header::FixedHeader, remaining_length::RemainingLength},
};

const PACKET_TYPE: u8 = 0x14;
const RESERVED_FIXED_HEADER_FLAGS: u8 = 0x00;

#[derive(Debug, Default)]
pub struct Disconnect;

impl Disconnect {
    pub fn new() -> Self {
        Self
    }

    pub fn from_bytes(fixed_header: FixedHeader) -> Result<Self, Error> {
        // Fixed Header
        let fixed_header_flags = fixed_header.first_byte() & 0b0000_1111;

        if fixed_header_flags != RESERVED_FIXED_HEADER_FLAGS {
            return Err(Error::new("Invalid reserved header flags".to_string()));
        }

        Ok(Disconnect::new())
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // Fixed Header
        let mut packet_bytes = vec![PACKET_TYPE << 4 | RESERVED_FIXED_HEADER_FLAGS];

        let remaining_length_value = 0_u32;
        let remaining_length_bytes = RemainingLength::new(remaining_length_value).to_bytes();
        packet_bytes.extend(remaining_length_bytes);

        packet_bytes
    }
}
