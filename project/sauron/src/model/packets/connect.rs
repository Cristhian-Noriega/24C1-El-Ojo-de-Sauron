use super::{CONNECT_PACKET_TYPE, RESERVED_FIXED_HEADER_FLAGS};
use crate::{
    EncodedString, Error, FixedHeader, Login, QoS, Read, RemainingLength, Will, PROTOCOL_LEVEL,
    PROTOCOL_NAME,
};

const VARIABLE_HEADER_LENGTH: usize = 10;

#[derive(Debug)]
pub struct Connect {
    // Variable Header Fields
    clean_session: bool,
    keep_alive: u16,

    // Payload Fields
    client_id: EncodedString,
    will: Option<Will>,
    login: Option<Login>,
}

impl Connect {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        clean_session: bool,
        keep_alive: u16,
        client_id: EncodedString,
        will: Option<Will>,
        login: Option<Login>,
    ) -> Self {
        Self {
            clean_session,
            keep_alive,
            client_id,
            will,
            login,
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

        for i in 0..PROTOCOL_NAME.len() {
            if variable_header_buffer[i] != PROTOCOL_NAME[i] {
                return Err(Error::new("Invalid protocol name".to_string()));
            }
        }

        let protocol_level_byte = variable_header_buffer[4];

        if protocol_level_byte != PROTOCOL_LEVEL {
            return Err(Error::new("Invalid protocol level".to_string()));
        }

        let flags_byte = variable_header_buffer[5];

        let reserved = flags_byte & 0b0000_0001;
        if reserved != 0 {
            return Err(Error::new("Invalid reserved flag".to_string()));
        }

        let clean_session = (flags_byte & 0b0000_0010) >> 1 == 1;
        let will_flag = (flags_byte & 0b0000_0100) >> 2 == 1;

        let will_qos = QoS::from_byte((flags_byte & 0b0001_1000) >> 3)?;
        if !will_flag && will_qos != QoS::AtMost {
            return Err(Error::new("Invalid will qos".to_string()));
        }

        let will_retain = (flags_byte & 0b0010_0000) >> 5 == 1;
        if !will_flag && will_retain {
            return Err(Error::new("Invalid will retain flag".to_string()));
        }

        let username_flag = (flags_byte & 0b1000_0000) >> 7 == 1;

        let password_flag = (flags_byte & 0b0100_0000) >> 6 == 1;
        if !username_flag && password_flag {
            return Err(Error::new("Invalid password flag".to_string()));
        }

        let keep_alive = u16::from_be_bytes([variable_header_buffer[8], variable_header_buffer[9]]);

        // Payload
        let client_id = EncodedString::from_bytes(stream)?;

        let will = if will_flag {
            Some(Will::from_bytes(stream, will_qos, will_retain)?)
        } else {
            None
        };

        let login = if username_flag {
            Some(Login::from_bytes(stream, password_flag)?)
        } else {
            None
        };

        Ok(Connect::new(
            clean_session,
            keep_alive,
            client_id,
            will,
            login,
        ))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // Payload
        let mut payload_bytes = vec![];

        payload_bytes.extend(self.client_id.to_bytes());

        if let Some(will) = &self.will {
            payload_bytes.extend(will.to_bytes());
        }

        if let Some(login) = &self.login {
            payload_bytes.extend(login.to_bytes());
        }

        // Variable Header
        let mut variable_header_bytes = vec![];
        variable_header_bytes.extend(PROTOCOL_NAME);
        variable_header_bytes.push(PROTOCOL_LEVEL);

        let (will_flag, will_qos, retain_flag) = match &self.will {
            Some(will) => (true, will.qos(), will.retain()),
            None => (false, &QoS::AtMost, false),
        };

        let (username_flag, password_flag) = match &self.login {
            Some(login) => (true, login.password().is_some()),
            None => (false, false),
        };

        let flags_byte = (self.clean_session as u8) << 1
            | (will_flag as u8) << 2
            | (will_qos.to_byte() << 3)
            | (retain_flag as u8) << 5
            | (password_flag as u8) << 6
            | (username_flag as u8) << 7;

        variable_header_bytes.push(flags_byte);
        variable_header_bytes.extend(&self.keep_alive.to_be_bytes());

        let mut fixed_header_bytes = vec![CONNECT_PACKET_TYPE << 4 | RESERVED_FIXED_HEADER_FLAGS];

        // Fixed Header
        let remaining_length_value =
            variable_header_bytes.len() as u32 + payload_bytes.len() as u32;
        let remaining_length_bytes = RemainingLength::new(remaining_length_value).to_bytes();
        fixed_header_bytes.extend(remaining_length_bytes);

        // Packet
        let mut packet_bytes = vec![];

        packet_bytes.extend(fixed_header_bytes);
        packet_bytes.extend(variable_header_bytes);
        packet_bytes.extend(payload_bytes);

        packet_bytes
    }
}
