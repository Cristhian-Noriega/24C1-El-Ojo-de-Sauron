use crate::{Error, Read};

const MAX_MULTIPLIER: u32 = u32::pow(128, 3);
const MAX_LENGTH: u32 = u32::pow(128, 4); // 268.435.455 bytes

#[derive(Debug)]
pub struct RemainingLength {
    value: u32,
}

impl RemainingLength {
    pub fn new(length: u32) -> RemainingLength {
        if length > MAX_LENGTH {
            return RemainingLength { value: MAX_LENGTH }; // TODO esto no está del todo bien
        }
        RemainingLength { value: length }
    }

    pub fn from_bytes(stream: &mut dyn Read) -> Result<Self, Error> {
        let mut multiplier = 1;
        let mut value = 0;

        loop {
            let mut buffer = [0];
            stream.read_exact(&mut buffer)?;

            let byte = buffer[0];
            value += (byte & 127) as u32 * multiplier;

            multiplier *= 128;

            if multiplier > MAX_MULTIPLIER {
                return Err(Error::new("Malformed remaining length".to_string()));
            }

            if byte & 128 == 0 {
                break;
            }
        }

        Ok(RemainingLength { value })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        let mut length = self.value;
        loop {
            let mut byte = self.value % 128;
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

    pub fn value(&self) -> usize {
        self.value as usize
    }

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
