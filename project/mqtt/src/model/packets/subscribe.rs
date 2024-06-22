use super::{DEFAULT_VARIABLE_HEADER_LENGTH, RESERVED_FIXED_HEADER_FLAGS, SUBSCRIBE_PACKET_TYPE};
use crate::{encrypt, Error, FixedHeader, QoS, Read, RemainingLength, TopicFilter};

/// Represents a SUBSCRIBE packet of MQTT. The client uses it to subscribe to one or more topics.
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

    /// Converts a byte stream into a Subscribe.
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

    /// Converts the Subscribe into a vector of bytes.
    pub fn to_bytes(&self, key: &[u8]) -> Vec<u8> {
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
        let mut fixed_header_bytes = vec![SUBSCRIBE_PACKET_TYPE << 4 | RESERVED_FIXED_HEADER_FLAGS];

        let remaining_length_value =
            variable_header_bytes.len() as u32 + payload_bytes.len() as u32;
        let remaining_length_bytes = RemainingLength::new(remaining_length_value).to_bytes();
        fixed_header_bytes.extend(remaining_length_bytes);

        let mut packet_bytes = vec![];

        packet_bytes.extend(fixed_header_bytes);
        packet_bytes.extend(variable_header_bytes);
        packet_bytes.extend(payload_bytes);

        encrypt(packet_bytes, key)
    }

    /// Returns the packet identifier.
    pub fn packet_identifier(&self) -> u16 {
        self.packet_identifier
    }

    /// Returns the topics to which the client subscribes.
    pub fn topics(&self) -> Vec<(TopicFilter, QoS)> {
        self.topics.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EncodedString;
    use std::io::Cursor;

    const KEY: &[u8; 32] = &[0; 32];

    #[allow(dead_code)]
    fn from_slice(bytes: &[u8]) -> impl Read {
        let encoded_string = EncodedString::new(bytes.to_vec());
        Cursor::new(encoded_string.to_bytes())
    }

    #[test]
    fn test_subscribe_from_bytes() {
        let packet_identifier = 1;
        let bytes = &mut from_slice(b"topic1");
        let topic_filter = TopicFilter::from_bytes(bytes).unwrap();

        let topics = vec![(topic_filter, QoS::AtMost)];

        let mut stream = std::io::Cursor::new(vec![
            0x00, 0x01, 0x00, 0x06, b't', b'o', b'p', b'i', b'c', b'1', 0x00,
        ]);

        let fixed_header = FixedHeader::new(SUBSCRIBE_PACKET_TYPE << 4, RemainingLength::new(11));
        let subscribe = Subscribe::from_bytes(fixed_header, &mut stream).unwrap();

        assert_eq!(subscribe.packet_identifier(), packet_identifier);
        assert_eq!(subscribe.topics(), topics);
    }

    #[test]
    fn test_subscribe_to_bytes() {
        let packet_identifier = 1;
        let bytes = &mut from_slice(b"topic1");
        let topic_filter = TopicFilter::from_bytes(bytes).unwrap();
        let topics = vec![(topic_filter, QoS::AtMost)];

        let subscribe = Subscribe::new(packet_identifier, topics);
        let bytes = subscribe.to_bytes(KEY);

        let expected_bytes = vec![
            128_u8, 0x0b, // Fixed Header
            0x00, 0x01, // Packet Identifier
            0x00, 6_u8, b't', b'o', b'p', b'i', b'c', b'1', 0x00, // Topic Filter
        ];
        let encrypted_bytes = encrypt(expected_bytes, KEY);

        assert_eq!(bytes, encrypted_bytes);
    }
}
