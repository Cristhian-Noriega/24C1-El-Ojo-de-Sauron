use std::fmt::{self, Display, Formatter};

use super::{PUBACK_PACKET_TYPE, RESERVED_FIXED_HEADER_FLAGS};
use crate::{Error, FixedHeader, Read, RemainingLength};

const PACKAGE_IDENTIFIER_LENGTH: usize = 2;

/// Represents a PUBACK packet from MQTT. The server uses it to confirm the reception of a PUBLISH packet.
#[derive(Debug, PartialEq)]
pub struct Puback {
    packet_identifier: Option<u16>,
}

impl Puback {
    pub fn new(packet_identifier: Option<u16>) -> Self {
        Self { packet_identifier }
    }

    /// Converts a stream of bytes into a Puback.
    pub fn from_bytes(fixed_header: FixedHeader, stream: &mut dyn Read) -> Result<Self, Error> {
        // Fixed Header
        let fixed_header_flags = fixed_header.first_byte() & 0b0000_1111;

        if fixed_header_flags != RESERVED_FIXED_HEADER_FLAGS {
            return Err(Error::new("Invalid fixed header flags".to_string()));
        }

        // Variable Header
        let mut packet_identifier_buffer = [0; PACKAGE_IDENTIFIER_LENGTH];
        stream.read_exact(&mut packet_identifier_buffer)?;

        let packet_identifier = Some(u16::from_be_bytes(packet_identifier_buffer));

        Ok(Puback::new(packet_identifier))
    }

    /// Converts the Puback into a vector of bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        // Variable Header
        let mut variable_header_bytes = vec![];

        // Split self.packet_identifier into bytes and push them to variable_header_bytes
        if let Some(packet_identifier) = self.packet_identifier {
            variable_header_bytes.extend_from_slice(&packet_identifier.to_be_bytes());
        }

        // Fixed Header
        let mut fixed_header_bytes = vec![PUBACK_PACKET_TYPE << 4 | RESERVED_FIXED_HEADER_FLAGS];

        let remaining_length_value = variable_header_bytes.len() as u32;
        let remaining_length_bytes = RemainingLength::new(remaining_length_value).to_bytes();
        fixed_header_bytes.extend(remaining_length_bytes);

        // Packet
        let mut packet_bytes = vec![];

        packet_bytes.extend(fixed_header_bytes);
        packet_bytes.extend(variable_header_bytes);

        packet_bytes
    }

    /// Returns the packet identifier.
    pub fn packet_identifier(&self) -> Option<u16> {
        self.packet_identifier
    }
}

impl Display for Puback {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let packet_identifier = match self.packet_identifier {
            Some(packet_identifier) => packet_identifier.to_string(),
            None => "None".to_string(),
        };
        write!(
            f,
            "Puback packet with packet identifier: {}",
            packet_identifier
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_puback_to_bytes() {
        let puback = Puback::new(Some(42));
        let expected_bytes: Vec<u8> = vec![0b0100_0000, 0x02, 0x00, 0x2A];
        assert_eq!(puback.to_bytes(), expected_bytes);
    }

    #[test]
    fn test_puback_from_bytes() {
        let mut stream = std::io::Cursor::new(vec![0x00, 0x2A]);
        let fixed_header = FixedHeader::new(0x4 << 4, RemainingLength::new(2));
        let puback = Puback::from_bytes(fixed_header, &mut stream).unwrap();
        assert_eq!(puback, Puback::new(Some(42)));
    }
}
