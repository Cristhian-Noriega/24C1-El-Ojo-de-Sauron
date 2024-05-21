use super::{CONNACK_PACKET_TYPE, DEFAULT_VARIABLE_HEADER_LENGTH, RESERVED_FIXED_HEADER_FLAGS};
use crate::{ConnackReturnCode, Error, FixedHeader, Read, RemainingLength};

#[derive(Debug)]
pub struct Connack {
    // Variable Header Fields
    session_present: bool,
    connect_return_code: ConnackReturnCode,
    // Connack no tiene payload
}

impl Connack {
    #[allow(clippy::too_many_arguments)]
    pub fn new(session_present: bool, connect_return_code: ConnackReturnCode) -> Self {
        Self {
            session_present,
            connect_return_code,
        }
    }

    pub fn from_bytes(fixed_header: FixedHeader, stream: &mut dyn Read) -> Result<Self, Error> {
        // Fixed Header
        let fixed_header_flags = fixed_header.first_byte() & 0b0000_1111;

        if fixed_header_flags != RESERVED_FIXED_HEADER_FLAGS {
            return Err(Error::new("Invalid flags".to_string()));
        }

        // Variable Header
        let mut variable_header_buffer = vec![0; DEFAULT_VARIABLE_HEADER_LENGTH];
        stream.read_exact(&mut variable_header_buffer)?;

        let connect_ack = variable_header_buffer[0];

        let session_present = (connect_ack & 0b0000_0001) == 0b0000_0001;

        let connect_return_code = ConnackReturnCode::from_byte(variable_header_buffer[1])?;

        Ok(Connack::new(session_present, connect_return_code))
    }

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
}
