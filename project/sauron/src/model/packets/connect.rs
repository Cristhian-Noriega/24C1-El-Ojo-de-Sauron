use super::{CONNECT_PACKET_TYPE, RESERVED_FIXED_HEADER_FLAGS};
use crate::{
    EncodedString, Error, FixedHeader, Login, QoS, Read, RemainingLength, Will, PROTOCOL_LEVEL,
    PROTOCOL_NAME,
};

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

        let protocol_name = EncodedString::from_bytes(stream)?;
        let protocol_name_content = protocol_name.content();

        for i in 0..PROTOCOL_NAME.len() {
            if protocol_name_content[i] != PROTOCOL_NAME[i] {
                return Err(Error::new("Invalid protocol name".to_string()));
            }
        }

        let protocol_level_buffer = &mut [0; 1];
        stream.read_exact(protocol_level_buffer)?;

        let protocol_level_byte = protocol_level_buffer[0];

        if protocol_level_byte != PROTOCOL_LEVEL {
            return Err(Error::new("Invalid protocol level".to_string()));
        }

        let flags_buffer = &mut [0; 1];
        stream.read_exact(flags_buffer)?;

        let flags_byte = flags_buffer[0];

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

        let keep_alive_buffer = &mut [0; 2];
        stream.read_exact(keep_alive_buffer)?;

        let keep_alive = u16::from_be_bytes(*keep_alive_buffer);

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

        let protocol_name = EncodedString::new(PROTOCOL_NAME.to_vec());
        variable_header_bytes.extend(protocol_name.to_bytes());

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

    pub fn clean_session(&self) -> bool {
        self.clean_session
    }

    pub fn keep_alive(&self) -> u16 {
        self.keep_alive
    }

    pub fn client_id(&self) -> &EncodedString {
        &self.client_id
    }

    pub fn will(&self) -> Option<&Will> {
        self.will.as_ref()
    }

    pub fn login(&self) -> Option<&Login> {
        self.login.as_ref()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{FixedHeader, TopicName};

    #[allow(dead_code)]
    fn fixed_header_bytes(remaining_length: RemainingLength) -> Vec<u8> {
        let fixed_header = FixedHeader::new(CONNECT_PACKET_TYPE << 4, remaining_length);

        fixed_header.to_bytes()
    }

    #[allow(dead_code)]
    fn variable_header_bytes(flags: u8, keep_alive: u16) -> Vec<u8> {
        let protocol_name_bytes = EncodedString::new(PROTOCOL_NAME.to_vec()).to_bytes();
        let protocol_level_byte = vec![PROTOCOL_LEVEL];

        let keep_alive_bytes = keep_alive.to_be_bytes();

        let mut variable_header_bytes = vec![];
        variable_header_bytes.extend(protocol_name_bytes);
        variable_header_bytes.extend(protocol_level_byte);
        variable_header_bytes.push(flags);
        variable_header_bytes.extend(keep_alive_bytes);

        variable_header_bytes
    }

    #[allow(dead_code)]
    fn header_bytes(remaining_length: RemainingLength, flags: u8, keep_alive: u16) -> Vec<u8> {
        let fixed_header_bytes = fixed_header_bytes(remaining_length);
        let variable_header_bytes = variable_header_bytes(flags, keep_alive);

        [&fixed_header_bytes[..], &variable_header_bytes[..]].concat()
    }

    #[test]
    fn test_simple_connect_to_bytes() {
        let clean_session = false;
        let keep_alive = 10;
        let client_id = EncodedString::new(b"a".to_vec());

        let connect = Connect::new(clean_session, keep_alive, client_id, None, None);
        let connect_bytes = connect.to_bytes();

        let expected_header_bytes = header_bytes(RemainingLength::new(13), 0b0000_0000, 10);
        let expected_payload_bytes = EncodedString::new(b"a".to_vec()).to_bytes();

        let expected_bytes = [&expected_header_bytes[..], &expected_payload_bytes[..]].concat();

        assert_eq!(connect_bytes, expected_bytes);
    }

    #[test]
    fn test_connect_to_bytes_with_will() {
        let clean_session = false;
        let keep_alive = 10;
        let client_id = EncodedString::new(b"a".to_vec());
        let will = Will::new(
            QoS::AtLeast,
            true,
            TopicName::new(vec![b"home".to_vec(), b"livingroom".to_vec()], false),
            EncodedString::new(b"message".to_vec()),
        );

        let expected_will_bytes = will.to_bytes();

        let connect = Connect::new(clean_session, keep_alive, client_id, Some(will), None);

        let connect_bytes = connect.to_bytes();

        let expected_header_bytes = header_bytes(RemainingLength::new(39), 0b0010_1100, 10);
        let expected_client_id_bytes = EncodedString::new(b"a".to_vec()).to_bytes();

        let expected_payload_bytes =
            [&expected_client_id_bytes[..], &expected_will_bytes[..]].concat();
        let expected_bytes = [&expected_header_bytes[..], &expected_payload_bytes[..]].concat();

        assert_eq!(connect_bytes, expected_bytes);
    }

    #[test]
    fn test_connect_to_bytes_with_login() {
        let clean_session = false;
        let keep_alive = 10;
        let client_id = EncodedString::new(b"a".to_vec());
        let login = Login::new(
            EncodedString::new(b"username".to_vec()),
            Some(EncodedString::new(b"password".to_vec())),
        );

        let expected_login_bytes = login.to_bytes();

        let connect = Connect::new(clean_session, keep_alive, client_id, None, Some(login));

        let connect_bytes = connect.to_bytes();

        let expected_header_bytes = header_bytes(RemainingLength::new(33), 0b1100_0000, 10);
        let expected_client_id_bytes = EncodedString::new(b"a".to_vec()).to_bytes();

        let expected_payload_bytes =
            [&expected_client_id_bytes[..], &expected_login_bytes[..]].concat();
        let expected_bytes = [&expected_header_bytes[..], &expected_payload_bytes[..]].concat();

        assert_eq!(connect_bytes, expected_bytes);
    }

    #[test]
    fn test_simple_connect_from_bytes() {
        let header_bytes = variable_header_bytes(0b0000_0000, 10);
        let client_id_bytes = EncodedString::new(b"a".to_vec()).to_bytes();

        let connect_bytes = [&header_bytes[..], &client_id_bytes[..]].concat();

        let fixed_header = FixedHeader::new(
            CONNECT_PACKET_TYPE << 4 | RESERVED_FIXED_HEADER_FLAGS,
            RemainingLength::new(13),
        );

        let connect = Connect::from_bytes(fixed_header, &mut connect_bytes.as_slice()).unwrap();

        assert!(!connect.clean_session());
        assert_eq!(connect.keep_alive(), 10);
        assert_eq!(connect.client_id(), &EncodedString::new(b"a".to_vec()));
        assert_eq!(connect.will(), None);
        assert_eq!(connect.login(), None);
    }

    #[test]
    fn test_connect_from_bytes_with_will() {
        let header_bytes = variable_header_bytes(0b0010_1100, 10);
        let client_id_bytes = EncodedString::new(b"a".to_vec()).to_bytes();

        let will = Will::new(
            QoS::AtLeast,
            true,
            TopicName::new(vec![b"home".to_vec(), b"livingroom".to_vec()], false),
            EncodedString::new(b"message".to_vec()),
        );

        let will_bytes = will.to_bytes();

        let connect_bytes = [&header_bytes[..], &client_id_bytes[..], &will_bytes[..]].concat();

        let fixed_header = FixedHeader::new(
            CONNECT_PACKET_TYPE << 4 | RESERVED_FIXED_HEADER_FLAGS,
            RemainingLength::new(39),
        );

        let connect = Connect::from_bytes(fixed_header, &mut connect_bytes.as_slice()).unwrap();

        assert_eq!(connect.will(), Some(&will));
    }

    #[test]
    fn test_connect_from_bytes_with_login() {
        let header_bytes = variable_header_bytes(0b1100_0000, 10);
        let client_id_bytes = EncodedString::new(b"a".to_vec()).to_bytes();

        let login = Login::new(
            EncodedString::new(b"username".to_vec()),
            Some(EncodedString::new(b"password".to_vec())),
        );

        let login_bytes = login.to_bytes();

        let connect_bytes = [&header_bytes[..], &client_id_bytes[..], &login_bytes[..]].concat();

        let fixed_header = FixedHeader::new(
            CONNECT_PACKET_TYPE << 4 | RESERVED_FIXED_HEADER_FLAGS,
            RemainingLength::new(33),
        );

        let connect = Connect::from_bytes(fixed_header, &mut connect_bytes.as_slice()).unwrap();

        assert_eq!(connect.login(), Some(&login));
    }

    #[test]
    fn test_invalid_fixed_header_flags() {
        let fixed_header = FixedHeader::new(
            CONNECT_PACKET_TYPE << 4 | 0b0000_0001,
            RemainingLength::new(13),
        );

        let connect_bytes = vec![];

        let connect = Connect::from_bytes(fixed_header, &mut connect_bytes.as_slice());
        assert!(connect.is_err());
    }

    #[test]
    fn test_invalid_protocol_name() {
        let invalid_protocol_name = vec![b'A', b'M', b'Q', b'P'];

        let connect_bytes = EncodedString::new(invalid_protocol_name).to_bytes();

        let fixed_header = FixedHeader::new(
            CONNECT_PACKET_TYPE << 4 | RESERVED_FIXED_HEADER_FLAGS,
            RemainingLength::new(13),
        );

        let connect = Connect::from_bytes(fixed_header, &mut connect_bytes.as_slice());
        assert!(connect.is_err());
    }

    #[test]
    fn test_invalid_protocol_level() {
        let protocol_name = EncodedString::new(PROTOCOL_NAME.to_vec()).to_bytes();
        let protocol_level = [0];

        let connect_bytes = [&protocol_name[..], &protocol_level[..]].concat();

        let fixed_header = FixedHeader::new(
            CONNECT_PACKET_TYPE << 4 | RESERVED_FIXED_HEADER_FLAGS,
            RemainingLength::new(13),
        );

        let connect = Connect::from_bytes(fixed_header, &mut connect_bytes.as_slice());
        assert!(connect.is_err());
    }

    #[test]
    fn test_invalid_flags() {
        let protocol_name_bytes = EncodedString::new(PROTOCOL_NAME.to_vec()).to_bytes();
        let protocol_level_byte = vec![PROTOCOL_LEVEL];

        let mut connect_bytes = vec![];
        connect_bytes.extend(protocol_name_bytes);
        connect_bytes.extend(protocol_level_byte);
        {
            let invalid_flags = 0b0000_0001;
            let mut connect_bytes = connect_bytes.clone();
            connect_bytes.push(invalid_flags);

            let fixed_header = FixedHeader::new(
                CONNECT_PACKET_TYPE << 4 | RESERVED_FIXED_HEADER_FLAGS,
                RemainingLength::new(13),
            );

            let connect = Connect::from_bytes(fixed_header, &mut connect_bytes.as_slice());
            assert!(connect.is_err());
        }

        {
            let invalid_flags = 0b1000_0000;
            let mut connect_bytes = connect_bytes.clone();
            connect_bytes.push(invalid_flags);

            let fixed_header = FixedHeader::new(
                CONNECT_PACKET_TYPE << 4 | RESERVED_FIXED_HEADER_FLAGS,
                RemainingLength::new(13),
            );

            let connect = Connect::from_bytes(fixed_header, &mut connect_bytes.as_slice());
            assert!(connect.is_err());
        }

        {
            let invalid_flags = 0b0000_1000;
            let mut connect_bytes = connect_bytes.clone();
            connect_bytes.push(invalid_flags);

            let fixed_header = FixedHeader::new(
                CONNECT_PACKET_TYPE << 4 | RESERVED_FIXED_HEADER_FLAGS,
                RemainingLength::new(13),
            );

            let connect = Connect::from_bytes(fixed_header, &mut connect_bytes.as_slice());
            assert!(connect.is_err());
        }

        {
            let invalid_flags = 0b0010_0000;
            let mut connect_bytes = connect_bytes.clone();
            connect_bytes.push(invalid_flags);

            let fixed_header = FixedHeader::new(
                CONNECT_PACKET_TYPE << 4 | RESERVED_FIXED_HEADER_FLAGS,
                RemainingLength::new(13),
            );

            let connect = Connect::from_bytes(fixed_header, &mut connect_bytes.as_slice());
            assert!(connect.is_err());
        }
    }
}
