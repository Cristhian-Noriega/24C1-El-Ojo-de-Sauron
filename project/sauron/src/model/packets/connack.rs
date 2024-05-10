use crate::errors::error::Error;
use crate::model::fixed_header::FixedHeader;
use crate::model::remaining_length::RemainingLength;
use crate::model::return_codes::connack_return_code::ConnackReturnCode;
use std::io::Read;

const RESERVED_FIXED_HEADER_FLAGS: u8 = 0x00;
const PACKET_TYPE: u8 = 0x02;

const VARIABLE_HEADER_LENGTH: usize = 2;

#[derive(Debug)]
pub struct Connack {
    // Variable Header Fields
    session_present_flag: bool,
    connect_return_code: ConnackReturnCode,
    // Connack no tiene payload
}

impl Connack {
    #[allow(clippy::too_many_arguments)]
    pub fn new(session_present_flag: bool, connect_return_code: ConnackReturnCode) -> Self {
        Self {
            session_present_flag,
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
        let mut variable_header_buffer = vec![0; VARIABLE_HEADER_LENGTH];
        stream.read_exact(&mut variable_header_buffer)?;

        let connect_ack_flags = variable_header_buffer[0];

        let session_present_flag = (connect_ack_flags & 0b0000_0001) == 0b0000_0001;

        let connect_return_code = ConnackReturnCode::from_byte(variable_header_buffer[1])?;

        Ok(Connack::new(session_present_flag, connect_return_code))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // Variable Header
        let mut variable_header_bytes = vec![];

        if self.session_present_flag {
            variable_header_bytes.push(0x01)
        } else {
            variable_header_bytes.push(0x00)
        }

        let connect_return_code_bytes = self.connect_return_code.to_byte();

        variable_header_bytes.push(connect_return_code_bytes);

        // Fixed Header
        let mut fixed_header_bytes = vec![PACKET_TYPE << 4 | RESERVED_FIXED_HEADER_FLAGS];

        let remaining_length_value = VARIABLE_HEADER_LENGTH as u32;
        let remaining_length_bytes = RemainingLength::new(remaining_length_value).to_bytes();
        fixed_header_bytes.extend(remaining_length_bytes);

        // Packet
        let mut packet_bytes = vec![];

        packet_bytes.extend(fixed_header_bytes);
        packet_bytes.extend(variable_header_bytes);

        packet_bytes
    }
}
