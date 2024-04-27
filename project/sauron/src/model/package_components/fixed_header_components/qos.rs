use crate::errors::error::Error;

pub enum QoS {
    AtMostOnce,
    AtLeastOnce,
    ExactlyOnce,
}

impl QoS {
    pub fn into_u8(self) -> u8 {
        match self {
            QoS::AtMostOnce => 0x00,
            QoS::AtLeastOnce => 0x01,
            QoS::ExactlyOnce => 0x02,
        }
    }

    pub fn from_u8(value: u8) -> Result<Self, Error> {
        match value {
            0x00 => Ok(QoS::AtMostOnce),
            0x01 => Ok(QoS::AtLeastOnce),
            0x02 => Ok(QoS::ExactlyOnce),
            _ => panic!("Invalid QoS value"),
        }
    }
}
