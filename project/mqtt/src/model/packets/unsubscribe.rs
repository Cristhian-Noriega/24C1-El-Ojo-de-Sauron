use super::{DEFAULT_VARIABLE_HEADER_LENGTH, RESERVED_FIXED_HEADER_FLAGS, UNSUBSCRIBE_PACKET_TYPE};
use crate::{encrypt, Error, FixedHeader, Read, RemainingLength, TopicFilter};

/// Represents an UNSUBSCRIBE packet from MQTT. The client uses it to unsubscribe from one or more topics.
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

    /// Converts a stream of bytes into an Unsubscribe.
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

    /// Converts the Unsubscribe into a vector of bytes.
    pub fn to_bytes(&self, key: &[u8]) -> Vec<u8> {
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

        let data_bytes = [&variable_header_bytes[..], &payload_bytes[..]].concat();
        let encrypted_bytes = encrypt(data_bytes, key);

        let mut packet_bytes = vec![];

        packet_bytes.extend(fixed_header_bytes);
        packet_bytes.extend(encrypted_bytes);

        packet_bytes
    }

    /// Returns the packet identifier.
    pub fn packet_identifier(&self) -> u16 {
        self.packet_identifier
    }

    /// Returns the topics to unsubscribe from.
    pub fn topics(&self) -> &Vec<TopicFilter> {
        &self.topics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encryptation::encryping_tool::decrypt;
    use crate::EncodedString;
    use crate::TopicFilter;
    use std::io::Cursor;

    const KEY: &[u8; 32] = &[0; 32];

    #[allow(dead_code)]
    fn from_slice(bytes: &[u8]) -> impl Read {
        let encoded_string = EncodedString::new(bytes.to_vec());
        Cursor::new(encoded_string.to_bytes())
    }

    #[test]
    fn test_unsubscribe_from_bytes() {
        let packet_identifier = 1;
        let bytes = &mut from_slice(b"topic1");
        let topic_filter = TopicFilter::from_bytes(bytes).unwrap();
        let topics = vec![topic_filter];

        let mut stream = std::io::Cursor::new(vec![
            0x00, 0x01, 0x00, 0x06, b't', b'o', b'p', b'i', b'c', b'1',
        ]);

        let fixed_header = FixedHeader::new(UNSUBSCRIBE_PACKET_TYPE << 4, RemainingLength::new(10));
        let unsubscribe = Unsubscribe::from_bytes(fixed_header, &mut stream).unwrap();

        assert_eq!(unsubscribe.packet_identifier(), packet_identifier);
        assert_eq!(unsubscribe.topics(), &topics);
    }

    #[test]
    fn test_unsubscribe_to_bytes() {
        let packet_identifier = 1;
        let bytes = &mut from_slice(b"topic1");
        let topic_filter = TopicFilter::from_bytes(bytes).unwrap();
        let topics = vec![topic_filter];

        let unsubscribe = Unsubscribe::new(packet_identifier, topics);
        let encrypted_bytes = unsubscribe.to_bytes(KEY);
        let fixed_header_bytes = &encrypted_bytes[0..2];
        let decrypted_bytes = decrypt(&encrypted_bytes[2..], KEY).unwrap();
        let unsubscribe_bytes = [&fixed_header_bytes[..], &decrypted_bytes[..]].concat();

        let expected_bytes = vec![
            160_u8, 10_u8, 0x00, 0x01, 0x00, 0x06, b't', b'o', b'p', b'i', b'c', b'1',
        ];

        assert_eq!(unsubscribe_bytes, expected_bytes);
    }
}
