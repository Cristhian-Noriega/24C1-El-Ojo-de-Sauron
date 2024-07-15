use std::fmt::{self, Display, Formatter};

use crate::{errors::error::MqttResult, Read};

const LENGTH_SIZE: usize = 2;

/// Represents an encoded string. Contains the length of the string and the content in a byte vector.
#[derive(Debug, PartialEq)]
pub struct EncodedString {
    length: u16,
    content: Vec<u8>,
}

impl EncodedString {
    pub fn new(content: Vec<u8>) -> Self {
        Self {
            length: content.len() as u16,
            content,
        }
    }

    /// Converts a byte stream into an EncodedString.
    pub fn from_bytes(stream: &mut dyn Read) -> MqttResult<Self> {
        let mut length_buffer = [0; LENGTH_SIZE];
        stream.read_exact(&mut length_buffer)?;

        let length = u16::from_be_bytes(length_buffer);

        let mut content = vec![0; length as usize];
        stream.read_exact(&mut content)?;

        Ok(Self { length, content })
    }

    /// Converts a string into an EncodedString.
    pub fn from_string(string: &String) -> Self {
        let length = string.len() as u16;
        let content = string.as_bytes().to_vec();

        Self { length, content }
    }

    /// Converts the EncodedString into a byte vector.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend(&self.length.to_be_bytes());
        bytes.extend(&self.content);

        bytes
    }

    /// Returns the length of the EncodedString.
    pub fn length(&self) -> usize {
        LENGTH_SIZE + self.length as usize
    }

    /// Returns the content of the EncodedString.
    pub fn content(&self) -> &Vec<u8> {
        &self.content
    }
}

impl Display for EncodedString {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let content = String::from_utf8_lossy(&self.content);
        write!(f, "{}", content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoded_string_new() {
        let content = vec![0x00, 0x01, 0x02, 0x03];
        let encoded_string = EncodedString::new(content.clone());

        assert_eq!(encoded_string.length, content.len() as u16);
        assert_eq!(encoded_string.content, content);
    }

    #[test]
    fn test_encoded_string_from_bytes() {
        let content = vec![0x00, 0x01, 0x02, 0x03];
        let mut bytes = vec![];
        bytes.extend(&(content.len() as u16).to_be_bytes());
        bytes.extend(&content);

        let mut stream = &bytes[..];
        let encoded_string = EncodedString::from_bytes(&mut stream).unwrap();

        assert_eq!(encoded_string.length, content.len() as u16);
        assert_eq!(encoded_string.content, content);
    }

    #[test]
    fn test_encoded_string_from_string() {
        let string = String::from("test");
        let encoded_string = EncodedString::from_string(&string);

        assert_eq!(encoded_string.length, string.len() as u16);
        assert_eq!(encoded_string.content, string.as_bytes());
    }

    #[test]
    fn test_encoded_string_to_bytes() {
        let content = vec![0x00, 0x01, 0x02, 0x03];
        let encoded_string = EncodedString::new(content.clone());

        let mut bytes = vec![];
        bytes.extend(&(content.len() as u16).to_be_bytes());
        bytes.extend(&content);

        assert_eq!(encoded_string.to_bytes(), bytes);
    }

    #[test]
    fn test_encoded_string_length() {
        let content = vec![0x00, 0x01, 0x02, 0x03];
        let encoded_string = EncodedString::new(content.clone());

        assert_eq!(encoded_string.length(), LENGTH_SIZE + content.len());
    }

    #[test]
    fn test_encoded_string_content() {
        let content = vec![0x00, 0x01, 0x02, 0x03];
        let encoded_string = EncodedString::new(content.clone());

        assert_eq!(encoded_string.content(), &content);
    }
}
