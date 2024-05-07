use crate::{
    errors::error::Error,
    model::{fixed_header::FixedHeader, topic_filter::TopicFilter},
};
use std::io::Read;

const PACKET_TYPE: u8 = 0x02;
const RESERVED_FIXED_HEADER_FLAGS: u8 = 0x02;

#[derive(Debug)]
pub struct Subscribe {
    pub packet_identifier: u16,
    pub topics: Vec<TopicFilter>,
}

impl Subscribe {
    pub fn new(packet_identifier: u16, topics: Vec<TopicFilter>) -> Self {
        Self {
            packet_identifier,
            topics,
        }
    }

    pub fn from_bytes(fixed_header: FixedHeader, stream: &mut dyn Read) -> Result<Self, Error> {
        // Fixed Header
        let fixed_header_flags = fixed_header.first_byte() & 0b0000_1111;

        if fixed_header_flags != RESERVED_FIXED_HEADER_FLAGS {
            return Err(Error::new("Invalid flags".to_string()));
        }

        // Variable Header
        let mut variable_header_buffer = [0; 2];
        stream.read_exact(&mut variable_header_buffer)?;

        let packet_identifier = u16::from_be_bytes(variable_header_buffer);

        // Payload
        let mut topics = Vec::new();
        loop {
            let topic_filter = TopicFilter::from_bytes(stream)?;
            topics.push(topic_filter);

            let mut buffer = [0; 1];
            stream.read_exact(&mut buffer)?;
            if buffer[0] == 0 {
                break;
            }
        }

        if topics.is_empty() {
            return Err(Error::new("No topics specified in the payload".to_string()));
        }

        Ok(Self {
            packet_identifier,
            topics,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Variable Header
        let packet_identifier_bytes = self.packet_identifier.to_be_bytes();
        bytes.extend_from_slice(&packet_identifier_bytes);

        // Payload
        for topic in &self.topics {
            bytes.extend_from_slice(&topic.topic_name.to_bytes());
            bytes.push(topic.qos.to_byte());
        }

        // Fixed Header
        let remaining_length = bytes.len();
        let fixed_header_bytes = vec![
            PACKET_TYPE << 4 | RESERVED_FIXED_HEADER_FLAGS,
            remaining_length as u8,
        ];

        let mut packet_bytes = vec![];

        packet_bytes.extend(fixed_header_bytes);
        packet_bytes.extend(bytes);

        packet_bytes
    }
}
