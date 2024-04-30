use crate::errors::error::Error;

pub enum QoS {
    AtMostOnce,
    AtLeastOnce,
    ExactlyOnce,
}

impl QoS {
    pub fn into_byte(self) -> u8 {
        match self {
            QoS::AtMostOnce => 0x00,
            QoS::AtLeastOnce => 0x01,
            QoS::ExactlyOnce => 0x02,
        }
    }

    pub fn from_byte(byte: u8) -> Result<Self, Error> {
        match byte {
            0x00 => Ok(QoS::AtMostOnce),
            0x01 => Ok(QoS::AtLeastOnce),
            0x02 => Ok(QoS::ExactlyOnce),
            _ => Err(Error::new(format!("Invalid QoS: {}", byte))),
        }
    }
}
