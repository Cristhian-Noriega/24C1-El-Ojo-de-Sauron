use crate::{MqttError, MqttResult, Read};

const MAX_MULTIPLIER: u32 = u32::pow(128, 3);
const MAX_LENGTH: u32 = u32::pow(128, 4); // 268.435.455 bytes

/// Represents the remaining length of an MQTT packet.
#[derive(Debug, PartialEq)]
pub struct RemainingLength {
    value: u32,
}

impl RemainingLength {
    pub fn new(length: u32) -> RemainingLength {
        if length > MAX_LENGTH {
            return RemainingLength { value: MAX_LENGTH };
        }
        RemainingLength { value: length }
    }

    /// Calculates the remaining length of an MQTT packet from a byte stream.
    pub fn from_bytes(stream: &mut dyn Read) -> MqttResult<RemainingLength> {
        let mut multiplier = 1;
        let mut value = 0;

        loop {
            let mut buffer = [0];
            stream.read_exact(&mut buffer)?;

            let byte = buffer[0];
            value += (byte & 127) as u32 * multiplier;

            multiplier *= 128;

            if multiplier > MAX_MULTIPLIER {
                return Err(MqttError::InvalidRemainingLength);
            }

            if byte & 128 == 0 {
                break;
            }
        }

        Ok(RemainingLength { value })
    }

    /// Converts the remaining length of an MQTT packet into a byte vector.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        let mut length = self.value;
        loop {
            let mut byte = length % 128;
            length /= 128;
            if length > 0 {
                byte |= 128;
            }
            bytes.push(byte as u8);
            if length == 0 {
                break;
            }
        }
        bytes
    }

    /// Returns the value of the remaining length.
    pub fn value(&self) -> usize {
        self.value as usize
    }

    /// Returns the length of the byte vector that represents the remaining length.
    pub fn length(&self) -> usize {
        let mut length = 0;
        let mut value = self.value;
        loop {
            value /= 128;
            length += 1;
            if value == 0 {
                break;
            }
        }
        length
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_remaining_length_to_bytes() {
        let remaining_length = RemainingLength::new(268_435_455);
        let bytes = remaining_length.to_bytes();
        assert_eq!(bytes, vec![0xFF, 0xFF, 0xFF, 0x7F]);
    }

    #[test]
    fn test_from_bytes_valid() {
        let data: Vec<u8> = vec![0x96, 0x01];
        let mut cursor = Cursor::new(data);
        let result = RemainingLength::from_bytes(&mut cursor).unwrap();
        assert_eq!(result, RemainingLength { value: 150 });
    }

    #[test]
    fn test_from_bytes_malformed_length() {
        let data: Vec<u8> = vec![0xFF, 0xFF, 0xFF, 0xFF, 0x07];
        let mut cursor = Cursor::new(data);
        let result = RemainingLength::from_bytes(&mut cursor);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_bytes_incomplete_stream() {
        let data: Vec<u8> = vec![0x96];
        let mut cursor = Cursor::new(data);
        let result = RemainingLength::from_bytes(&mut cursor);
        assert!(result.is_err());
    }
}
