use super::{DEFAULT_VARIABLE_HEADER_LENGTH, RESERVED_FIXED_HEADER_FLAGS, UNSUBACK_PACKET_TYPE};
use crate::{encrypt, FixedHeader, MqttError, MqttResult, Read, RemainingLength};

/// Represents an UNSUBACK packet from MQTT. The server uses it to confirm the unsubscription of one or more topics.
#[derive(Debug)]
pub struct Unsuback {
    packet_identifier: u16,
}

impl Unsuback {
    pub fn new(packet_identifier: u16) -> Self {
        Self { packet_identifier }
    }

    /// Converts a stream of bytes into an Unsuback.
    pub fn from_bytes(fixed_header: FixedHeader, stream: &mut dyn Read) -> MqttResult<Self> {
        // Fixed Header
        let fixed_header_flags = fixed_header.first_byte() & 0b0000_1111;

        if fixed_header_flags != RESERVED_FIXED_HEADER_FLAGS {
            return Err(MqttError::InvalidFixedHeaderFlags);
        }

        // Variable Header
        let mut variable_header_buffer = [0; DEFAULT_VARIABLE_HEADER_LENGTH];
        stream.read_exact(&mut variable_header_buffer)?;

        let packet_identifier =
            u16::from_be_bytes([variable_header_buffer[0], variable_header_buffer[1]]);

        Ok(Unsuback::new(packet_identifier))
    }

    /// Converts the Unsuback into a vector of bytes.
    pub fn to_bytes(&self, key: &[u8]) -> Vec<u8> {
        // Variable Header
        let variable_header_bytes = self.packet_identifier.to_be_bytes().to_vec();

        // Fixed Header
        let mut fixed_header_bytes = vec![UNSUBACK_PACKET_TYPE << 4 | RESERVED_FIXED_HEADER_FLAGS];

        let remaining_length_value = variable_header_bytes.len() as u32;
        let remaining_length_bytes = RemainingLength::new(remaining_length_value).to_bytes();
        fixed_header_bytes.extend(remaining_length_bytes);

        let encrypted_bytes = match encrypt(variable_header_bytes, key) {
            Ok(bytes) => bytes,
            Err(_) => return vec![],
        };

        let mut packet_bytes = vec![];

        packet_bytes.extend(fixed_header_bytes);
        packet_bytes.extend(encrypted_bytes);

        packet_bytes
    }

    /// Returns the packet identifier.
    pub fn packet_identifier(&self) -> u16 {
        self.packet_identifier
    }
}

#[cfg(test)]
mod tests {
    use crate::encryptation::encryping_tool::decrypt;

    use super::*;

    const KEY: &[u8; 32] = &[0; 32];

    #[test]
    fn test_unsuback_to_bytes() {
        let unsuback = Unsuback::new(42);
        let encrypted_bytes = unsuback.to_bytes(KEY);
        let fixed_header_bytes = encrypted_bytes[0..2].to_vec();
        let decrypted_bytes = decrypt(&encrypted_bytes[2..], KEY).unwrap();
        let unsuback_bytes = [&fixed_header_bytes[..], &decrypted_bytes[..]].concat();

        let expected_bytes: Vec<u8> = vec![0b1011_0000, 0x02, 0x00, 0x2A];

        assert_eq!(unsuback_bytes, expected_bytes);
    }

    #[test]
    fn test_unsuback_from_bytes() {
        let bytes: Vec<u8> = vec![0x00, 0x2A];

        let fixed_header = FixedHeader::new(176_u8, RemainingLength::new(2));
        let unsuback = Unsuback::from_bytes(fixed_header, &mut bytes.as_slice()).unwrap();

        assert_eq!(unsuback.packet_identifier(), 42);
    }
}
