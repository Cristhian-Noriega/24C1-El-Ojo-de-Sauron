use std::fmt::{Display, Formatter};

use super::{CONNACK_PACKET_TYPE, DEFAULT_VARIABLE_HEADER_LENGTH, RESERVED_FIXED_HEADER_FLAGS};
use crate::{ConnectReturnCode, Error, FixedHeader, Read, RemainingLength};

/// Represents a CONNECT packet of MQTT that is used to accept a connection from a client.
#[derive(Debug)]
pub struct Connack {
    // Variable Header Fields
    session_present: bool,
    connect_return_code: ConnectReturnCode,
    // Connack no tiene payload
}

impl Connack {
    #[allow(clippy::too_many_arguments)]
    pub fn new(session_present: bool, connect_return_code: ConnectReturnCode) -> Self {
        Self {
            session_present,
            connect_return_code,
        }
    }

    /// Converts a stream of bytes into a Connack.
    pub fn from_bytes(fixed_header: FixedHeader, stream: &mut dyn Read) -> Result<Self, Error> {
        // Fixed Header
        let fixed_header_flags = fixed_header.first_byte() & 0b0000_1111;

        if fixed_header_flags != RESERVED_FIXED_HEADER_FLAGS {
            return Err(Error::new("Invalid flags".to_string()));
        }

        // Variable Header
        let mut variable_header_buffer = [0; DEFAULT_VARIABLE_HEADER_LENGTH];
        stream.read_exact(&mut variable_header_buffer)?;

        let connect_ack = variable_header_buffer[0];

        let session_present = (connect_ack & 0b0000_0001) == 0b0000_0001;

        let connect_return_code = ConnectReturnCode::from_byte(variable_header_buffer[1])?;

        Ok(Connack::new(session_present, connect_return_code))
    }

    /// Converts the Connack into a vector of bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        // Variable Header
        let mut variable_header_bytes = vec![];

        let session_present = if self.session_present { 0x01 } else { 0x00 };

        variable_header_bytes.push(session_present);

        let connect_return_code_bytes = self.connect_return_code.to_byte();

        variable_header_bytes.push(connect_return_code_bytes);

        // Fixed Header
        let mut fixed_header_bytes = vec![CONNACK_PACKET_TYPE << 4 | RESERVED_FIXED_HEADER_FLAGS];

        let remaining_length_value = DEFAULT_VARIABLE_HEADER_LENGTH as u32;
        let remaining_length_bytes = RemainingLength::new(remaining_length_value).to_bytes();
        fixed_header_bytes.extend(remaining_length_bytes);

        // Packet
        let mut packet_bytes = vec![];

        packet_bytes.extend(fixed_header_bytes);
        packet_bytes.extend(variable_header_bytes);

        packet_bytes
    }

    /// Returns if the session is present.
    pub fn session_present(&self) -> bool {
        self.session_present
    }

    /// Returns the connection return code.
    pub fn connect_return_code(&self) -> &ConnectReturnCode {
        &self.connect_return_code
    }
}

impl Display for Connack {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Connack packet with return code: {:?} and sessionPresent: {}",
            self.connect_return_code(),
            self.session_present()
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::FixedHeader;

    #[allow(dead_code)]
    fn fixed_header_bytes() -> Vec<u8> {
        let remaining_length = RemainingLength::new(DEFAULT_VARIABLE_HEADER_LENGTH as u32);
        let fixed_header = FixedHeader::new(CONNACK_PACKET_TYPE << 4, remaining_length);

        fixed_header.to_bytes()
    }

    #[allow(dead_code)]
    fn variable_header_bytes(
        session_present: bool,
        connect_return_code: ConnectReturnCode,
    ) -> Vec<u8> {
        let session_present_byte = if session_present { 0x01 } else { 0x00 };
        let connect_return_code_byte = connect_return_code.to_byte();

        vec![session_present_byte, connect_return_code_byte]
    }

    #[allow(dead_code)]
    fn header_bytes(session_present: bool, connect_return_code: ConnectReturnCode) -> Vec<u8> {
        let fixed_header_bytes = fixed_header_bytes();
        let variable_header_bytes = variable_header_bytes(session_present, connect_return_code);

        [&fixed_header_bytes[..], &variable_header_bytes[..]].concat()
    }

    #[test]
    fn test_simple_connack_to_bytes() {
        let session_present = false;
        let connect_return_code = ConnectReturnCode::ConnectionAccepted;

        let connack = Connack::new(session_present, connect_return_code);
        let connack_bytes = connack.to_bytes();

        let connect_return_code = ConnectReturnCode::ConnectionAccepted;
        let expected_bytes = header_bytes(session_present, connect_return_code);

        assert_eq!(connack_bytes, expected_bytes);
    }

    #[test]
    fn test_simple_connack_from_bytes() {
        let connack_bytes = variable_header_bytes(false, ConnectReturnCode::ConnectionAccepted);
        let fixed_header = FixedHeader::new(CONNACK_PACKET_TYPE << 4, RemainingLength::new(2));

        let connack = Connack::from_bytes(fixed_header, &mut connack_bytes.as_slice()).unwrap();

        assert!(!connack.session_present());
        assert_eq!(
            connack.connect_return_code(),
            &ConnectReturnCode::ConnectionAccepted
        );
    }

    #[test]
    fn test_invalid_fixed_header_flags() {
        let fixed_header = FixedHeader::new(
            CONNACK_PACKET_TYPE << 4 | 0b0000_0001,
            RemainingLength::new(2),
        );

        let connack_bytes = vec![];

        let connack = Connack::from_bytes(fixed_header, &mut connack_bytes.as_slice());
        assert!(connack.is_err());
    }

    #[test]
    fn test_invalid_return_code() {
        let connack_bytes = vec![0b0000_0000, 0b0000_0110];
        let fixed_header = FixedHeader::new(CONNACK_PACKET_TYPE << 4, RemainingLength::new(2));

        let connack = Connack::from_bytes(fixed_header, &mut connack_bytes.as_slice());
        assert!(connack.is_err());
    }
}
