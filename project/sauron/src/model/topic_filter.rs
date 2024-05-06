use std::io::Read;

use crate::{
    errors::error::Error,
    model::{encoded_string::EncodedString, qos::QoS},
};

#[derive(Debug)]
pub struct TopicFilter {
    pub name: EncodedString,
    pub qos: QoS,
}

impl TopicFilter {
    pub fn new(name: EncodedString, qos: QoS) -> Self {
        Self { name, qos }
    }

    pub fn from_bytes(stream: &mut dyn Read) -> Result<Self, Error> {
        let name = EncodedString::from_bytes(stream)?;
        if !Self::is_valid_topic_name(&name) {
            return Err(Error::new("Invalid topic name".to_string()));
        }
        let qos_buffer = &mut [0; 1];
        stream.read_exact(qos_buffer)?;
        let qos = QoS::from_byte(qos_buffer[0])?;

        Ok(Self { name, qos })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&self.name.to_bytes());
        bytes.push(self.qos.to_byte());

        bytes
    }

    pub fn is_valid_topic_name(name: &EncodedString) -> bool {
        let content = &name.content();
        if content.is_empty() {
            return false;
        }
        if content.starts_with(&[b'/']) || content.ends_with(&[b'/']) {
            return false;
        }
        let levels: Vec<&[u8]> = content.split(|&byte| byte == b'/').collect();
        for (i, level) in levels.iter().enumerate() {
            if level.contains(&b'+') && level.len() > 1 {
                return false;
            }
            if level.contains(&b'#')
                && (level.len() > 1 || i != levels.len() - 1 || !level.ends_with(&[b'#']))
            {
                return false;
            }
        }
        true
    }
}

#[allow(unused_imports)]
mod test {
    use super::*;
    use crate::model::encoded_string::EncodedString;
    use crate::model::qos::QoS;
    use std::io::Cursor;

    #[test]
    fn test_is_valid_topic_name() {
        let content = b"home/livingroom".to_vec();
        let length = content.len() as u16;
        let encoded_string = EncodedString::new(length, content);
        assert_eq!(TopicFilter::is_valid_topic_name(&encoded_string), true);

        let content = b"home/living room".to_vec();
        let length = content.len() as u16;
        let encoded_string = EncodedString::new(length, content);
        assert_eq!(TopicFilter::is_valid_topic_name(&encoded_string), true);

        let content = b"home/+/livingroom".to_vec();
        let length = content.len() as u16;
        let encoded_string = EncodedString::new(length, content);
        assert_eq!(TopicFilter::is_valid_topic_name(&encoded_string), true);

        let content = b"+/+/+/#".to_vec();
        let length = content.len() as u16;
        let encoded_string = EncodedString::new(length, content);
        assert_eq!(TopicFilter::is_valid_topic_name(&encoded_string), true);
    }

    #[test]
    fn test_from_bytes_valid() {
        let mut buffer = Cursor::new(vec![0x00, 0x04, b't', b'e', b's', b't', 0x00]);
        let topic_filter = TopicFilter::from_bytes(&mut buffer).unwrap();
        assert_eq!(topic_filter.name.content(), &[b't', b'e', b's', b't']);
        assert_eq!(topic_filter.qos, QoS::AtMost);
    }

    #[test]
    fn test_from_bytes_invalid() {
        let mut stream = Cursor::new(b"\x00\x04tes");
        assert!(EncodedString::from_bytes(&mut stream).is_err());
    }

    #[test]
    fn test_to_bytes() {
        let content = vec![b't', b'e', b's', b't'];
        let length = content.len() as u16;
        let encoded_string = EncodedString::new(length, content.clone());

        let bytes = encoded_string.to_bytes();

        assert_eq!(bytes, b"\x00\x04test"); // Expected byte sequence
    }

    #[test]
    fn test_topic_name_empty_invalid() {
        let content = vec![];
        let length = content.len() as u16;
        let encoded_string = EncodedString::new(length, content);
        assert_eq!(TopicFilter::is_valid_topic_name(&encoded_string), false);
    }

    #[test]
    fn test_topic_name_starts_with_slash_invalid() {
        let content = vec![b'/'];
        let length = content.len() as u16;
        let encoded_string = EncodedString::new(length, content);
        assert_eq!(TopicFilter::is_valid_topic_name(&encoded_string), false);
    }

    #[test]
    fn test_invalid_topic_name_single_level_wildcard() {
        let content = b"home+/livingroom".to_vec();
        let length = content.len() as u16;
        let encoded_string = EncodedString::new(length, content);
        assert_eq!(TopicFilter::is_valid_topic_name(&encoded_string), false);

        let content = b"home+".to_vec();
        let length = content.len() as u16;
        let encoded_string = EncodedString::new(length, content);
        assert_eq!(TopicFilter::is_valid_topic_name(&encoded_string), false);

        let content = b"+home/livingroom".to_vec();
        let length = content.len() as u16;
        let encoded_string = EncodedString::new(length, content);
        assert_eq!(TopicFilter::is_valid_topic_name(&encoded_string), false);

        let content = b"home/livin+groom".to_vec();
        let length = content.len() as u16;
        let encoded_string = EncodedString::new(length, content);
        assert_eq!(TopicFilter::is_valid_topic_name(&encoded_string), false);
    }
}
