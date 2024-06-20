use super::{PINGRESP_PACKET_TYPE, RESERVED_FIXED_HEADER_FLAGS};
use crate::{Error, FixedHeader, RemainingLength};

/// Represents a PINGRESP packet from MQTT. The server responds to the client's PING request.
#[derive(Debug, Default, PartialEq)]
pub struct Pingresp;

impl Pingresp {
    pub fn new() -> Self {
        Self
    }

    /// Converts a stream of bytes into a Pingresp.
    pub fn from_bytes(fixed_header: FixedHeader) -> Result<Self, Error> {
        // Fixed Header
        let fixed_header_flags = fixed_header.first_byte() & 0b0000_1111;

        if fixed_header_flags != RESERVED_FIXED_HEADER_FLAGS {
            return Err(Error::new("Invalid reserved header flags".to_string()));
        }

        Ok(Pingresp::new())
    }

    /// Converts the Pingresp into a vector of bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        // Fixed Header
        let mut packet_bytes = vec![PINGRESP_PACKET_TYPE << 4 | RESERVED_FIXED_HEADER_FLAGS];

        let remaining_length_value = 0;
        let remaining_length_bytes = RemainingLength::new(remaining_length_value).to_bytes();
        packet_bytes.extend(remaining_length_bytes);

        packet_bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pingresp_to_bytes() {
        let pingresp = Pingresp::new();
        let expected_bytes: Vec<u8> = vec![0b1101_0000, 0x00];
        assert_eq!(pingresp.to_bytes(), expected_bytes);
    }

    #[test]
    fn test_pingresp_from_bytes() {
        let remaining_length = RemainingLength::new(2_u32);
        let fixed_header = FixedHeader::new(0xD << 4, remaining_length);
        let pingresp = Pingresp::from_bytes(fixed_header).unwrap();
        assert_eq!(pingresp, Pingresp::new());
    }
}