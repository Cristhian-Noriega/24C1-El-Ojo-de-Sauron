use std::io::Read;

use crate::errors::error::Error;

use super::{encoded_string::EncodedString, topic_level::TopicLevel};

const FORWARD_SLASH: u8 = 0x2F;

#[derive(Debug)]
pub struct TopicName {
    levels: Vec<Vec<u8>>,
}

impl TopicName {
    pub fn new(levels: Vec<Vec<u8>>) -> Self {
        Self { levels }
    }

    pub fn from_bytes(stream: &mut dyn Read) -> Result<Self, Error> {
        let encoded_string_topic_name = EncodedString::from_bytes(stream)?;
        let bytes = encoded_string_topic_name.content();

        if bytes.is_empty() {
            return Err(Error::new("Invalid topic name".to_string()));
        }

        let levels_bytes: Vec<Vec<u8>> = bytes
            .split(|&byte| byte == FORWARD_SLASH)
            .map(|slice| slice.to_vec())
            .collect();

        let mut levels = vec![];

        for level in levels_bytes {
            match TopicLevel::from_bytes(level)? {
                TopicLevel::Literal(level) => levels.push(level),
                _ => return Err(Error::new("Wildcard not allowed in topic name".to_string())),
            }
        }

        Ok(Self { levels })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let topic_bytes = self
            .levels
            .iter()
            .map(|level| level.to_vec())
            .chain(std::iter::once(vec![FORWARD_SLASH]))
            .flatten()
            .collect();

        EncodedString::new(topic_bytes).to_bytes()
    }

    pub fn levels(&self) -> &Vec<Vec<u8>> {
        &self.levels
    }

    pub fn length(&self) -> usize {
        self.to_bytes().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[allow(dead_code)]
    fn from_slice(bytes: &[u8]) -> impl Read {
        let encoded_string = EncodedString::new(bytes.to_vec());
        Cursor::new(encoded_string.to_bytes())
    }

    #[test]
    fn test_valid_topic_names() {
        let bytes = &mut from_slice(b"home/livingroom");
        assert!(TopicName::from_bytes(bytes).is_ok());

        let bytes = &mut from_slice(b"/");
        assert!(TopicName::from_bytes(bytes).is_ok());
    }

    #[test]
    fn test_invalid_topic_names() {
        let bytes = &mut from_slice(b"home/+/livingroom");
        assert!(TopicName::from_bytes(bytes).is_err());

        let bytes = &mut from_slice(b"home/livingroom/#");
        assert!(TopicName::from_bytes(bytes).is_err());

        let bytes = &mut from_slice(b"home/livingroom#");
        assert!(TopicName::from_bytes(bytes).is_err());

        let bytes = &mut from_slice(b"+home/livingroom");
        assert!(TopicName::from_bytes(bytes).is_err());
    }

    #[test]
    fn test_length() {
        let bytes = &mut from_slice(b"home/livingroom");
        let topic_name = TopicName::from_bytes(bytes).unwrap();
        assert_eq!(topic_name.length(), 17);

        let bytes = &mut from_slice(b"/");
        let topic_name = TopicName::from_bytes(bytes).unwrap();
        assert_eq!(topic_name.length(), 3);
    }
}
