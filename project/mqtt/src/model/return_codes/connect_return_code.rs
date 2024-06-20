use crate::Error;

/// Representa los distintos códigos de retorno de una conexión MQTT.
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
    /// Convierte el código de retorno de conexión en un byte.
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

    /// Convierte un byte en un código de retorno de conexión.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connect_return_code_to_byte() {
        assert_eq!(ConnectReturnCode::ConnectionAccepted.to_byte(), 0x00);
        assert_eq!(ConnectReturnCode::UnacceptableProtocolVersion.to_byte(), 0x01);
        assert_eq!(ConnectReturnCode::IdentifierRejected.to_byte(), 0x02);
        assert_eq!(ConnectReturnCode::ServerUnavailable.to_byte(), 0x03);
        assert_eq!(ConnectReturnCode::BadUsernameOrPassword.to_byte(), 0x04);
        assert_eq!(ConnectReturnCode::NotAuthorized.to_byte(), 0x05);
    }

    #[test]
    fn test_connect_return_code_from_byte() {
        assert_eq!(
            ConnectReturnCode::from_byte(0x00).unwrap(),
            ConnectReturnCode::ConnectionAccepted
        );
        assert_eq!(
            ConnectReturnCode::from_byte(0x01).unwrap(),
            ConnectReturnCode::UnacceptableProtocolVersion
        );
        assert_eq!(
            ConnectReturnCode::from_byte(0x02).unwrap(),
            ConnectReturnCode::IdentifierRejected
        );
        assert_eq!(
            ConnectReturnCode::from_byte(0x03).unwrap(),
            ConnectReturnCode::ServerUnavailable
        );
        assert_eq!(
            ConnectReturnCode::from_byte(0x04).unwrap(),
            ConnectReturnCode::BadUsernameOrPassword
        );
        assert_eq!(
            ConnectReturnCode::from_byte(0x05).unwrap(),
            ConnectReturnCode::NotAuthorized
        );
    }
}
