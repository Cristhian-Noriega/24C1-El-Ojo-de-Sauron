use std::io::Read;

use crate::errors::error::Error;

pub struct EncodedString {
    length: u16,
    content: Vec<u8>,
}

impl EncodedString {
    pub fn new(length: u16, content: Vec<u8>) -> Self {
        Self { length, content }
    }

    pub fn into_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![];
        bytes.extend(&self.length.to_be_bytes());
        bytes.extend(&self.content);

        bytes
    }

    pub fn from_bytes(stream: &mut dyn Read) -> Result<Self, Error> {
        let mut length_buffer = [0; 2];
        stream.read_exact(&mut length_buffer)?;

        let length = u16::from_be_bytes(length_buffer);

        let mut content = vec![0; length as usize];
        stream.read_exact(&mut content)?;

        Ok(Self { length, content })
    }

    pub fn get_length(&self) -> usize {
        self.length as usize
    }

    pub fn get_content(&self) -> &Vec<u8> {
        &self.content
    }
}
