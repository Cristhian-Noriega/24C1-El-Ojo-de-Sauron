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
        if content.iter().any(|&byte| byte == b'+' || byte == b'#') {
            return false;
        }
        true
    }
}
