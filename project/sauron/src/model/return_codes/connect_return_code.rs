use crate::Error;

#[derive(PartialEq, Debug)]
pub enum ConnectReturnCode {
    ConnectionAccepted,
    UnacceptableProtocolVersion,
    IdentifierRejected,
    ServerUnavailable,
    BadUsernameOrPassword,
    NotAuthorized,
}

impl ConnectReturnCode {
    pub fn to_byte(&self) -> u8 {
        match self {
            ConnectReturnCode::ConnectionAccepted => 0x00,
            ConnectReturnCode::UnacceptableProtocolVersion => 0x01,
            ConnectReturnCode::IdentifierRejected => 0x02,
            ConnectReturnCode::ServerUnavailable => 0x03,
            ConnectReturnCode::BadUsernameOrPassword => 0x04,
            ConnectReturnCode::NotAuthorized => 0x05,
        }
    }

    pub fn from_byte(byte: u8) -> Result<Self, Error> {
        match byte {
            0x00 => Ok(ConnectReturnCode::ConnectionAccepted),
            0x01 => Ok(ConnectReturnCode::UnacceptableProtocolVersion),
            0x02 => Ok(ConnectReturnCode::IdentifierRejected),
            0x03 => Ok(ConnectReturnCode::ServerUnavailable),
            0x04 => Ok(ConnectReturnCode::BadUsernameOrPassword),
            0x05 => Ok(ConnectReturnCode::NotAuthorized),
            _ => Err(Error::new(format!("Invalid ConnackReturnCode: {}", byte))),
        }
    }
}
