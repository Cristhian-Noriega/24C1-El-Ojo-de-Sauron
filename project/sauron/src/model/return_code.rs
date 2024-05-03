use crate::errors::error::Error;

#[derive(PartialEq, Debug)]
pub enum ReturnCode {
    ConnectionAccepted,
    UnacceptableProtocolVersion,
    IdentifierRejected,
    ServerUnavailable,
    BadUsernameOrPassword,
    NotAuthorized
}

impl ReturnCode {
    pub fn to_byte(&self) -> u8 {
        match self {
            ReturnCode::ConnectionAccepted => 0x00,
            ReturnCode::UnacceptableProtocolVersion => 0x01,
            ReturnCode::IdentifierRejected => 0x02,
            ReturnCode::ServerUnavailable => 0x03,
            ReturnCode::BadUsernameOrPassword => 0x04,
            ReturnCode::NotAuthorized => 0x05
        }
    }

    pub fn from_byte(byte: u8) -> Result<Self, Error> {
        match byte {
            0x00 => Ok(ReturnCode::ConnectionAccepted),
            0x01 => Ok(ReturnCode::UnacceptableProtocolVersion),
            0x02 => Ok(ReturnCode::IdentifierRejected),
            0x03 => Ok(ReturnCode::ServerUnavailable),
            0x04 => Ok(ReturnCode::BadUsernameOrPassword),
            0x05 => Ok(ReturnCode::NotAuthorized),
            _ => Err(Error::new(format!("Invalid ReturnCode: {}", byte))),
        }
    }
}