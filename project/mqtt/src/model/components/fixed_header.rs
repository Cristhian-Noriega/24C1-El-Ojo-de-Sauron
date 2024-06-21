use crate::{Error, Read, RemainingLength};

/// Represents the fixed header of an MQTT packet.
pub struct FixedHeader {
    first_byte: u8,
    remaining_length: RemainingLength,
}

impl FixedHeader {
    pub fn new(first_byte: u8, remaining_length: RemainingLength) -> FixedHeader {
        FixedHeader {
            first_byte,
            remaining_length,
        }
    }

    /// Converts a byte stream into a FixedHeader.
    pub fn from_bytes(stream: &mut dyn Read) -> Result<FixedHeader, Error> {
        let first_byte_buffer = &mut [0; 1];
        stream.read_exact(first_byte_buffer)?;

        let first_byte = first_byte_buffer[0];
        let remaining_length = RemainingLength::from_bytes(stream)?;

        Ok(FixedHeader {
            first_byte,
            remaining_length,
        })
    }

    /// Converts the FixedHeader into a byte vector.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut fixed_header_bytes = vec![self.first_byte];
        fixed_header_bytes.extend(self.remaining_length.to_bytes());

        fixed_header_bytes
    }

    /// Returns the first byte of the fixed header.
    pub fn first_byte(&self) -> u8 {
        self.first_byte
    }

    /// Returns the remaining length of the fixed header.
    pub fn remaining_length(&self) -> &RemainingLength {
        &self.remaining_length
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_header_to_bytes() {
        let first_byte = 0b0001_0000;
        let remaining_length = RemainingLength::new(10);
        let fixed_header = FixedHeader::new(first_byte, remaining_length);

        let bytes = fixed_header.to_bytes();

        assert_eq!(bytes, vec![0b0001_0000, 10]);
    }

    #[test]
    fn test_fixed_header_from_bytes() {
        let bytes = vec![0b0001_0000, 10];
        let mut stream = std::io::Cursor::new(bytes);

        let fixed_header = FixedHeader::from_bytes(&mut stream).unwrap();

        assert_eq!(fixed_header.first_byte(), 0b0001_0000);
        assert_eq!(fixed_header.remaining_length().value(), 10);
    }
}