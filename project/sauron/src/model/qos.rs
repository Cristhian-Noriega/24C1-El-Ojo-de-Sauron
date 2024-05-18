use crate::errors::error::Error;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub enum QoS {
    AtMost,
    AtLeast,
    Exactly,
}

impl QoS {
    pub fn to_byte(&self) -> u8 {
        match self {
            QoS::AtMost => 0x00,
            QoS::AtLeast => 0x01,
            QoS::Exactly => 0x02,
        }
    }

    pub fn from_byte(byte: u8) -> Result<Self, Error> {
        match byte {
            0x00 => Ok(QoS::AtMost),
            0x01 => Ok(QoS::AtLeast),
            0x02 => Ok(QoS::Exactly),
            _ => Err(Error::new(format!("Invalid QoS: {}", byte))),
        }
    }
}
