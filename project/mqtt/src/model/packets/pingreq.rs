use super::{PINGREQ_PACKET_TYPE, RESERVED_FIXED_HEADER_FLAGS};
use crate::{Error, FixedHeader, RemainingLength};

/// Represents a PINGREQ packet from MQTT. The client sends a PING request to the server.
#[derive(Debug, Default, PartialEq)]
pub struct Pingreq;

impl Pingreq {
    pub fn new() -> Self {
        Self
    }

    /// Converts a stream of bytes into a Pingreq.
    pub fn from_bytes(fixed_header: FixedHeader) -> Result<Self, Error> {
        // Fixed Header
        let fixed_header_flags = fixed_header.first_byte() & 0b0000_1111;

        if fixed_header_flags != RESERVED_FIXED_HEADER_FLAGS {
            return Err(Error::new("Invalid reserved header flags".to_string()));
        }

        Ok(Pingreq::new())
    }

    /// Converts the Pingreq into a vector of bytes.
    pub fn to_bytes(&self, _key: &[u8]) -> Vec<u8> {
        // Fixed Header
        let mut packet_bytes = vec![PINGREQ_PACKET_TYPE << 4 | RESERVED_FIXED_HEADER_FLAGS];

        let remaining_length_value = 0;
        let remaining_length_bytes = RemainingLength::new(remaining_length_value).to_bytes();
        packet_bytes.extend(remaining_length_bytes);

        packet_bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: &[u8; 32] = &[0; 32];

    #[test]
    fn test_pingreq_to_bytes() {
        let pingreq = Pingreq::new();
        let bytes = pingreq.to_bytes(KEY);

        let expected_bytes: Vec<u8> = vec![0b1100_0000, 0x00];

        assert_eq!(bytes, expected_bytes);
    }

    #[test]
    fn test_pingreq_from_bytes() {
        let remaining_length = RemainingLength::new(2_u32);
        let fixed_header = FixedHeader::new(0xC << 4, remaining_length);
        let pingreq = Pingreq::from_bytes(fixed_header).unwrap();
        assert_eq!(pingreq, Pingreq::new());
    }
}
