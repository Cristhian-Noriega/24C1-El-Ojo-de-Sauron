use crate::{Error, Read};

const LENGTH_SIZE: usize = 2;

#[derive(Debug, PartialEq)]
pub struct EncodedString {
    length: u16,
    pub content: Vec<u8>,
}

impl EncodedString {
    pub fn new(content: Vec<u8>) -> Self {
        Self {
            length: content.len() as u16,
            content,
        }
    }

    pub fn from_bytes(stream: &mut dyn Read) -> Result<Self, Error> {
        let mut length_buffer = [0; LENGTH_SIZE];
        stream.read_exact(&mut length_buffer)?;

        let length = u16::from_be_bytes(length_buffer);

        let mut content = vec![0; length as usize];
        stream.read_exact(&mut content)?;

        Ok(Self { length, content })
    }

    pub fn from_string(string: &String) -> Self {
        let length = string.len() as u16;
        let content = string.as_bytes().to_vec();

        Self { length, content }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend(&self.length.to_be_bytes());
        bytes.extend(&self.content);

        bytes
    }

    pub fn length(&self) -> usize {
        LENGTH_SIZE + self.length as usize
    }

    pub fn content(&self) -> &Vec<u8> {
        &self.content
    }
}
