use crate::errors::error::Error;
use std::io::Read;

const FIXED_HEADER_LENGTH: usize = 2;
const RESERVED_FIXED_HEADER_FLAGS: u8 = 0x00;
const PACKET_TYPE: u8 = 0x02;

const CONNECK_ACK_FLAGS_LENGTH: usize = 1;
const CONNECK_RETURN_CODE_LENGTH: usize = 1;

#[derive(Debug)]
pub struct ConnackPacket {
    // Variable Header Fields
    session_present_flag: bool,
    connect_return_code: u8,

    // Connack no tiene payload
}

impl ConnackPacket {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        session_present_flag: bool,
        connect_return_code: u8
    ) -> Self {
        Self {
            session_present_flag,
            connect_return_code,
        }
    }

    pub fn from_bytes(stream: &mut dyn Read) -> Result<Self, Error> {
        // Fixed Header
        let mut fixed_buffer = [0; FIXED_HEADER_LENGTH];
        stream.read_exact(&mut fixed_buffer)?;

        let first_byte = fixed_buffer[0];

        if first_byte >> 4 != PACKET_TYPE {
            return Err(Error::new("Invalid control packet type".to_string()));
        }

        if first_byte & 0b0000_1111 != RESERVED_FIXED_HEADER_FLAGS {
            return Err(Error::new("Invalid flags".to_string()));
        }

        // Variable Header
        let mut connect_ack_flags_buffer = [0; CONNECK_ACK_FLAGS_LENGTH];
        stream.read_exact(&mut connect_ack_flags_buffer)?;

        let connect_ack_flags_byte = connect_ack_flags_buffer[0];
        let session_present_flag: bool;

        if (connect_ack_flags_byte & 0b0000_0001) == 0b0000_0001 {
            session_present_flag = true
        } else {
            session_present_flag = false
        }

        let mut connect_return_code_buffer = [0; CONNECK_RETURN_CODE_LENGTH];
        stream.read_exact(&mut connect_return_code_buffer)?;

        let connect_return_code_byte = connect_return_code_buffer[0];

        Ok(ConnackPacket::new(
            session_present_flag,
            connect_return_code_byte,
        ))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // Variable Header
        let mut variable_header_bytes = vec![];

        if self.session_present_flag{
            variable_header_bytes.push(0x01)
        } else {
            variable_header_bytes.push(0x00)
        }

        variable_header_bytes.push(self.connect_return_code);

        // Fixed Header
        let remaining_length = variable_header_bytes.len();

        let fixed_header_bytes = vec![
            PACKET_TYPE << 4 | RESERVED_FIXED_HEADER_FLAGS,
            remaining_length as u8,
        ];

        // Packet
        let mut packet_bytes = vec![];

        packet_bytes.extend(fixed_header_bytes);
        packet_bytes.extend(variable_header_bytes);

        packet_bytes
    }
}
