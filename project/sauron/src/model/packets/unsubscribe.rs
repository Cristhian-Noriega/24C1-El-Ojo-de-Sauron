use super::{DEFAULT_VARIABLE_HEADER_LENGTH, RESERVED_FIXED_HEADER_FLAGS, UNSUBSCRIBE_PACKET_TYPE};
use crate::{Error, FixedHeader, Read, RemainingLength, TopicFilter};

#[derive(Debug)]
pub struct Unsubscribe {
    // Variable Header
    packet_identifier: u16,

    // Payload
    topics: Vec<TopicFilter>,
}

impl Unsubscribe {
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
        let mut variable_header_buffer = [0; DEFAULT_VARIABLE_HEADER_LENGTH];
        stream.read_exact(&mut variable_header_buffer)?;

        let packet_identifier = u16::from_be_bytes(variable_header_buffer);

        let mut remaining_length =
            fixed_header.remaining_length().value() - DEFAULT_VARIABLE_HEADER_LENGTH;

        // Payload
        let mut topics = vec![];
        while remaining_length > 0 {
            let topic_filter = TopicFilter::from_bytes(stream)?;
            remaining_length -= topic_filter.length();

            topics.push(topic_filter);
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

        for topic_filter in &self.topics {
            payload_bytes.extend(topic_filter.to_bytes());
        }

        // Fixed Header
        let mut fixed_header_bytes =
            vec![UNSUBSCRIBE_PACKET_TYPE << 4 | RESERVED_FIXED_HEADER_FLAGS];

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
