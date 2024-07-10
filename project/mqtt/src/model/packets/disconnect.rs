use super::{DISCONNECT_PACKET_TYPE, RESERVED_FIXED_HEADER_FLAGS};
use crate::{MqttResult, MqttError, FixedHeader, RemainingLength};

/// Represents a DISCONNECT packet in MQTT. The client uses it to disconnect from the server.
#[derive(Debug, Default, PartialEq)]
pub struct Disconnect;

impl Disconnect {
    pub fn new() -> Self {
        Self
    }

    /// Converts a stream of bytes into a Disconnect.
    pub fn from_bytes(fixed_header: FixedHeader) -> MqttResult<Self> {
        // Fixed Header
        let fixed_header_flags = fixed_header.first_byte() & 0b0000_1111;

        if fixed_header_flags != RESERVED_FIXED_HEADER_FLAGS {
            return Err(MqttError::InvalidFixedHeaderFlags)
        }

        Ok(Disconnect::new())
    }

    /// Converts the Disconnect into a vector of bytes.
    pub fn to_bytes(&self, _key: &[u8]) -> Vec<u8> {
        // Fixed Header
        let mut packet_bytes = vec![DISCONNECT_PACKET_TYPE << 4 | RESERVED_FIXED_HEADER_FLAGS];

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
    fn test_disconnect_to_bytes() {
        let disconnect = Disconnect::new();
        let bytes = disconnect.to_bytes(KEY);

        let expected_bytes: Vec<u8> = vec![0b1110_0000, 0x00];

        assert_eq!(bytes, expected_bytes);
    }

    #[test]
    fn test_disconnect_from_bytes() {
        let remaining_length = RemainingLength::new(2_u32);
        let fixed_header = FixedHeader::new(0xE << 4, remaining_length);
        let disconnect = Disconnect::from_bytes(fixed_header).unwrap();
        assert_eq!(disconnect, Disconnect::new());
    }
}
