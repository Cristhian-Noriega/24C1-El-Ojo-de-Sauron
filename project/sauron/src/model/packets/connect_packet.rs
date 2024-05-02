use crate::{errors::error::Error, model::encoded_string::EncodedString, model::qos::QoS};
use std::io::Read;

const FIXED_HEADER_LENGTH: usize = 2;
const RESERVED_FIXED_HEADER_FLAGS: u8 = 0x00;
const PACKET_TYPE: u8 = 0x01;

const PROTOCOL_LEVEL_LENGTH: usize = 1;
const PROTOCOL_NAME: [u8; 4] = [b'M', b'Q', b'T', b'T'];
const PROTOCOL_LEVEL: u8 = 0x04;
const FLAGS_LENGTH: usize = 1;
const KEEP_ALIVE_LENGTH: usize = 2;

#[derive(Debug)]
pub struct ConnectPacket {
    // Variable Header Fields
    clean_session: bool,
    keep_alive: u16,

    // Payload Fields
    client_id: EncodedString,
    will: Option<(QoS, bool, EncodedString, EncodedString)>, // tendría un struct will
    user: Option<(EncodedString, Option<EncodedString>)>,    // tendría un struct user
}

impl ConnectPacket {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        clean_session: bool,
        keep_alive: u16,
        client_id: EncodedString,
        will: Option<(QoS, bool, EncodedString, EncodedString)>,
        user: Option<(EncodedString, Option<EncodedString>)>,
    ) -> Self {
        Self {
            clean_session,
            keep_alive,
            client_id,
            will,
            user,
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
        for letter in PROTOCOL_NAME.iter() {
            let mut letter_buffer = [0; 1];
            stream.read_exact(&mut letter_buffer)?;

            if letter_buffer[0] != *letter {
                return Err(Error::new("Invalid protocol name".to_string()));
            }
        }

        let mut protocol_level_buffer = [0; PROTOCOL_LEVEL_LENGTH];
        stream.read_exact(&mut protocol_level_buffer)?;

        if protocol_level_buffer[0] != PROTOCOL_LEVEL {
            return Err(Error::new("Invalid protocol level".to_string()));
        }

        let flags_buffer = &mut [0; FLAGS_LENGTH];
        stream.read_exact(flags_buffer)?;

        let reserved = flags_buffer[0] & 0b0000_0001;
        if reserved != 0 {
            return Err(Error::new("Invalid reserved flag".to_string()));
        }

        let clean_session = (flags_buffer[0] & 0b0000_0010) >> 1 == 1;
        let will_flag = (flags_buffer[0] & 0b0000_0100) >> 2 == 1;

        let will_qos = QoS::from_byte((flags_buffer[0] & 0b0001_1000) >> 3)?;
        if !will_flag && will_qos != QoS::AtMost {
            return Err(Error::new("Invalid will qos".to_string()));
        }

        let will_retain = (flags_buffer[0] & 0b0010_0000) >> 5 == 1;
        if !will_flag && will_retain {
            return Err(Error::new("Invalid will retain flag".to_string()));
        }

        let username_flag = (flags_buffer[0] & 0b1000_0000) >> 7 == 1;

        let password_flag = (flags_buffer[0] & 0b0100_0000) >> 6 == 1;
        if !username_flag && password_flag {
            return Err(Error::new("Invalid password flag".to_string()));
        }

        let mut keep_alive_buffer = [0; KEEP_ALIVE_LENGTH];
        stream.read_exact(&mut keep_alive_buffer)?;

        let keep_alive = u16::from_be_bytes(keep_alive_buffer);
        //

        // Payload
        let client_id = EncodedString::from_bytes(stream)?;

        let will = if will_flag {
            let will_topic = EncodedString::from_bytes(stream)?;
            let will_message = EncodedString::from_bytes(stream)?;

            Some((will_qos, will_retain, will_topic, will_message))
        } else {
            None
        };

        let user = if username_flag {
            let username = EncodedString::from_bytes(stream)?;

            let password = if password_flag {
                Some(EncodedString::from_bytes(stream)?)
            } else {
                None
            };

            Some((username, password))
        } else {
            None
        };

        Ok(ConnectPacket::new(
            clean_session,
            keep_alive,
            client_id,
            will,
            user,
        ))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // Payload
        let mut payload_bytes: Vec<u8> = vec![];

        payload_bytes.extend(self.client_id.to_bytes());

        if let Some((_, _, will_topic, will_message)) = &self.will {
            payload_bytes.extend(will_topic.to_bytes());
            payload_bytes.extend(will_message.to_bytes());
        }

        if let Some((username, password)) = &self.user {
            payload_bytes.extend(username.to_bytes());

            if let Some(password) = password {
                payload_bytes.extend(password.to_bytes());
            }
        }

        // Variable Header
        let mut variable_header_bytes = vec![];
        variable_header_bytes.extend(PROTOCOL_NAME);
        variable_header_bytes.push(PROTOCOL_LEVEL);

        let (will_flag, will_qos, retain_flag) = match &self.will {
            Some((qos, retain, _, _)) => (true, qos, *retain),
            None => (false, &QoS::AtMost, false),
        };

        let (username_flag, password_flag) = match &self.user {
            Some((_, password)) => (true, password.is_some()),
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

        // Fixed Header
        let remaining_length = variable_header_bytes.len() + payload_bytes.len();

        let fixed_header_bytes = vec![
            PACKET_TYPE << 4 | RESERVED_FIXED_HEADER_FLAGS,
            remaining_length as u8,
        ];

        // Packet
        let mut packet_bytes = vec![];

        packet_bytes.extend(fixed_header_bytes);
        packet_bytes.extend(variable_header_bytes);
        packet_bytes.extend(payload_bytes);

        packet_bytes
    }
}
