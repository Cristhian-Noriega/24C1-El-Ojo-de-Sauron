use crate::errors::error::Error;
pub enum ControlPacketType {
    Connect,
}

impl ControlPacketType {
    pub fn to_byte(&self) -> u8 {
        match self {
            ControlPacketType::Connect => 0x01,
        }
    }

    pub fn from_byte(value: u8) -> Result<Self, Error> {
        match value {
            0x01 => Ok(ControlPacketType::Connect),
            _ => Err(Error::new(format!(
                "Invalid Control Packet Type: {}",
                value
            ))),
        }
    }
}
