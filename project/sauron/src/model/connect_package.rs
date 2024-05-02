use std::io::Read;
use crate::{model::encoded_string::EncodedString, errors::error::Error, model::qos::QoS, model::constants::*};

pub struct ConnectPackage {
    // Fixed Header Fields
    control_packet_type: u8,
    flags: u8,
    remaining_length: usize,

    // Variable Header Fields
    packet_identifier: u16,
    username_flag: bool,
    password_flag: bool,
    will_retain: bool,
    will_qos: QoS,
    will_flag: bool,
    clean_session: bool,
    keep_alive: u16,

    // Payload Fields
    client_id: EncodedString,
    will: Option<(EncodedString, EncodedString)>,
    user: Option<(EncodedString, Option<EncodedString>)>,
}

impl ConnectPackage {
    #[allow(clippy::too_many_arguments)] pub fn new(
        remaining_length: usize,
        packet_identifier: u16,
        username_flag: bool,
        password_flag: bool,
        will_retain: bool,
        will_qos: QoS,
        will_flag: bool,
        clean_session: bool,
        keep_alive: u16,
        client_id: EncodedString,
        will: Option<(EncodedString, EncodedString)>,
        user: Option<(EncodedString, Option<EncodedString>)>, 
    ) -> Self {
        Self {
            control_packet_type: 0x01,
            flags: 0x00,
            remaining_length,
            packet_identifier,
            username_flag,
            password_flag,
            will_retain,
            will_qos,
            will_flag,
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

        // let first_byte = buffer[0]; ya no es necesario

        stream.read_exact(&mut fixed_buffer)?;
        let mut remaining_length = fixed_buffer[1] as usize;


        // Variable Header
        let variable_buffer = &mut [0; PACKAGE_IDENTIFIER_LENGTH];
        stream.read_exact(variable_buffer)?;

        let packet_identifier = u16::from_be_bytes(*variable_buffer);

        let mut variable_buffer2 = [0; CONNECT_LENGTH];
        stream.read_exact(&mut variable_buffer2)?;

        for letter in PROTOCOL_NAME.iter() {
            if *letter != variable_buffer2[0] {
                return Err(Error::new("Invalid protocol name".to_string()));
            }
        }

        if variable_buffer2[4] != PROTOCOL_LEVEL {
            return Err(Error::new("Invalid protocol level".to_string()));
        }

        let flags_byte = variable_buffer2[5];

        if (flags_byte & 0b0000_0001) != 0 {
            return Err(Error::new("Invalid connect flags".to_string()));
        }

        let clean_session = (flags_byte & 0b0000_0010) >> 1 == 1;
        let will_flag = (flags_byte & 0b0000_0100) >> 2 == 1;
        let will_qos = QoS::from_byte((flags_byte & 0b0001_1000) >> 3)?;
        let will_retain = (flags_byte & 0b0010_0000) >> 5 == 1;

        if !will_flag && will_qos != QoS::AtMost {
            return Err(Error::new("Invalid will qos".to_string()));
        }

        if !will_flag && will_retain {
            return Err(Error::new("Invalid will retain flag".to_string()));
        }

        let username_flag = (flags_byte & 0b1000_0000) >> 7 == 1;
        let password_flag = (flags_byte & 0b0100_0000) >> 6 == 1;

        if !username_flag && password_flag {
            return Err(Error::new("Invalid password flag".to_string()));
        }

        let keep_alive = u16::from_be_bytes([variable_buffer2[6], variable_buffer2[7]]);

        remaining_length -= PACKAGE_IDENTIFIER_LENGTH + CONNECT_LENGTH;


        // Payload
        let client_id = EncodedString::from_bytes(stream)?;

        let will = if will_flag {
            let will_topic = EncodedString::from_bytes(stream)?;
            let will_message = EncodedString::from_bytes(stream)?;

            Some((will_topic, will_message))
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

        Ok(ConnectPackage::new(
            remaining_length,
            packet_identifier,
            username_flag,
            password_flag,
            will_retain,
            will_qos,
            will_flag,
            clean_session,
            keep_alive,
            client_id,
            will,
            user
        ))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut package_bytes: Vec<u8> = vec![];

        // Payload
        let mut payload_bytes = vec![];

        payload_bytes.extend(self.client_id.to_bytes());

        if let Some((will_topic, will_message)) = &self.will {
            payload_bytes.extend(will_topic.to_bytes());
            payload_bytes.extend(will_message.to_bytes());
        }

        if let Some((username, password)) = &self.user {
            payload_bytes.extend(username.to_bytes());

            if let Some(password) = password {
                payload_bytes.extend(password.to_bytes());
            }
        }

        package_bytes.extend(payload_bytes);

        // Variable Header
        let mut variable_header_bytes = vec![];
        let packet_identifier_bytes = self.packet_identifier.to_be_bytes();
        variable_header_bytes.extend(packet_identifier_bytes);

        let flags_byte = (self.clean_session as u8) << 1
            | (self.will_flag as u8) << 2
            | (self.will_qos.to_byte() << 3)
            | (self.will_retain as u8) << 5
            | (self.password_flag as u8) << 6
            | (self.username_flag as u8) << 7;

        variable_header_bytes.extend(PROTOCOL_NAME);
        variable_header_bytes.push(PROTOCOL_LEVEL);
        variable_header_bytes.push(flags_byte);
        variable_header_bytes.extend(&self.keep_alive.to_be_bytes());

        package_bytes.extend(variable_header_bytes);

        // Fixed Header
        let fixed_header_bytes = vec![
            self.control_packet_type << 4 | self.flags,
            self.remaining_length as u8,
        ];
        
        package_bytes.extend(fixed_header_bytes);

        package_bytes
    }
}
