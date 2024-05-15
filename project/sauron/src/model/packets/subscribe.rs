use crate::{
    errors::error::Error,
    model::{
        fixed_header::FixedHeader, qos::QoS, remaining_length::RemainingLength,
        topic_filter::TopicFilter,
    },
};
use std::io::Read;

const PACKET_TYPE: u8 = 0x08;
const RESERVED_FIXED_HEADER_FLAGS: u8 = 0x02;

#[derive(Debug)]
pub struct Subscribe {
    packet_identifier: u16,
    topics: Vec<(TopicFilter, QoS)>,
}

impl Subscribe {
    pub fn new(packet_identifier: u16, topics: Vec<(TopicFilter, QoS)>) -> Self {
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
        let mut remaining_length = fixed_header.remaining_length().value() - 2; // Del variable header

        while remaining_length > 0 {
            let topic_filter = TopicFilter::from_bytes(stream)?;

            let qos_buffer = &mut [0; 1];
            stream.read_exact(qos_buffer)?;
            let qos = QoS::from_byte(qos_buffer[0])?;

            remaining_length -= topic_filter.length() + 1; // Del qos

            topics.push((topic_filter, qos));
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
        // Variable Header
        let mut variable_header_bytes = vec![];

        let packet_identifier_bytes = self.packet_identifier.to_be_bytes();
        variable_header_bytes.extend_from_slice(&packet_identifier_bytes);

        // Payload
        let mut payload_bytes = vec![];

        for (topic_filter, qos) in &self.topics {
            payload_bytes.extend(topic_filter.to_bytes());
            payload_bytes.push(qos.to_byte());
        }

        // Fixed Header
        let mut fixed_header_bytes = vec![PACKET_TYPE << 4 | RESERVED_FIXED_HEADER_FLAGS];

        let remaining_length_value =
            variable_header_bytes.len() as u32 + payload_bytes.len() as u32;
        let remaining_length_bytes = RemainingLength::new(remaining_length_value).to_bytes();
        fixed_header_bytes.extend(remaining_length_bytes);

        let mut packet_bytes = vec![];

        packet_bytes.extend(fixed_header_bytes);
        packet_bytes.extend(variable_header_bytes);

        packet_bytes
    }
}
