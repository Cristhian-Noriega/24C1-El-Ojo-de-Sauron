use crate::Error;

#[derive(PartialEq, Debug)]
pub enum SubackReturnCode {
    SuccessMaximumQoS0,
    SuccessMaximumQoS1,
    SuccessMaximumQoS2,
    Failure,
}

impl SubackReturnCode {
    pub fn to_byte(&self) -> u8 {
        match self {
            SubackReturnCode::SuccessMaximumQoS0 => 0x00,
            SubackReturnCode::SuccessMaximumQoS1 => 0x01,
            SubackReturnCode::SuccessMaximumQoS2 => 0x02,
            SubackReturnCode::Failure => 0x80,
        }
    }

    pub fn from_byte(byte: u8) -> Result<Self, Error> {
        match byte {
            0x00 => Ok(SubackReturnCode::SuccessMaximumQoS0),
            0x01 => Ok(SubackReturnCode::SuccessMaximumQoS1),
            0x02 => Ok(SubackReturnCode::SuccessMaximumQoS2),
            0x80 => Ok(SubackReturnCode::Failure),
            _ => Err(Error::new(format!("Invalid SubackReturnCode: {}", byte))),
        }
    }
}
