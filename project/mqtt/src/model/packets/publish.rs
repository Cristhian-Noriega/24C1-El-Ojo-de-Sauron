use super::{DEFAULT_VARIABLE_HEADER_LENGTH, PUBLISH_PACKET_TYPE};
use crate::{Error, FixedHeader, QoS, Read, RemainingLength, TopicName};

#[derive(Debug, Clone)]
pub struct Publish {
    dup: bool,
    qos: QoS,
    retain: bool,
    topic: TopicName,
    package_identifier: Option<u16>,
    message: Vec<u8>,
}

impl Publish {
    pub fn new(
        dup: bool,
        qos: QoS,
        retain: bool,
        topic: TopicName,
        package_identifier: Option<u16>,
        message: Vec<u8>,
    ) -> Self {
        Self {
            dup,
            qos,
            retain,
            topic,
            package_identifier,
            message,
        }
    }

    pub fn from_bytes(fixed_header: FixedHeader, stream: &mut dyn Read) -> Result<Self, Error> {
        // Fixed Header

        let fixed_header_flags = fixed_header.first_byte() & 0b0000_1111;

        let dup = (fixed_header_flags >> 3) & 1 == 1;
        let qos = QoS::from_byte((fixed_header_flags >> 1) & 0b11)?;
        let retain = fixed_header_flags & 1 == 1;

        let remaining_length = fixed_header.remaining_length().value();

        // Variable Header

        let topic = TopicName::from_bytes(stream)?;

        let package_identifier = match qos {
            QoS::AtMost => None,
            _ => {
                let mut package_identifier_buffer = [0; DEFAULT_VARIABLE_HEADER_LENGTH];
                stream.read_exact(&mut package_identifier_buffer)?;

                Some(u16::from_be_bytes(package_identifier_buffer))
            }
        };

        let variable_header_len =
            topic.length() + package_identifier.map_or(0, |_| DEFAULT_VARIABLE_HEADER_LENGTH);

        // Payload

        let payload_len = remaining_length - variable_header_len;

        let mut message = vec![0; payload_len];
        stream.read_exact(&mut message)?;

        Ok(Publish::new(
            dup,
            qos,
            retain,
            topic,
            package_identifier,
            message,
        ))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // Payload
        let payload_bytes = &self.message;

        // Variable Header

        let mut variable_header_bytes = vec![];

        variable_header_bytes.extend(self.topic.to_bytes());

        if let Some(package_identifier) = self.package_identifier {
            variable_header_bytes.extend(&package_identifier.to_be_bytes());
        }

        // Fixed Header

        let fixed_header_flags = (if self.dup { 1 } else { 0 } << 3)
            | (self.qos.to_byte() << 1)
            | (if self.retain { 1 } else { 0 });

        let mut fixed_header_bytes = vec![PUBLISH_PACKET_TYPE << 4 | fixed_header_flags];

        let remaining_length_value =
            variable_header_bytes.len() as u32 + payload_bytes.len() as u32;
        let remaining_length_bytes = RemainingLength::new(remaining_length_value).to_bytes();
        fixed_header_bytes.extend(remaining_length_bytes);

        let mut packet_bytes = vec![];

        packet_bytes.extend(fixed_header_bytes);
        packet_bytes.extend(variable_header_bytes);
        packet_bytes.extend(payload_bytes);

        packet_bytes
    }

    pub fn dup(&self) -> bool {
        self.dup
    }

    pub fn qos(&self) -> &QoS {
        &self.qos
    }

    pub fn retain(&self) -> bool {
        self.retain
    }

    pub fn topic(&self) -> &TopicName {
        &self.topic
    }

    pub fn package_identifier(&self) -> Option<u16> {
        self.package_identifier
    }

    pub fn message(&self) -> &Vec<u8> {
        &self.message
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EncodedString;
    use std::io::Cursor;

    #[allow(dead_code)]
    fn from_slice(bytes: &[u8]) -> impl Read {
        let encoded_string = EncodedString::new(bytes.to_vec());
        Cursor::new(encoded_string.to_bytes())
    }

    #[test]
    fn test_from_bytes() {
        let mut stream = std::io::Cursor::new(vec![
            0b0011_0000, 6 as u8, 0x00, 0x03, b'a', b'/', b'b', b'c'
        ]);

        let fixed_header = FixedHeader::from_bytes(&mut stream).unwrap();
        let publish = Publish::from_bytes(fixed_header, &mut stream).unwrap();

        assert_eq!(publish.dup(), false);
        assert_eq!(publish.qos(), &QoS::AtMost);
        assert_eq!(publish.retain(), false);
        assert_eq!(publish.topic().to_string(), "a/b");
        assert_eq!(publish.package_identifier(), None);
        assert_eq!(publish.message(), &vec![b'c']);
    }

    #[test]
    fn test_to_bytes() {
        let bytes = &mut from_slice(b"a/b");
        let topic_name = TopicName::from_bytes(bytes).unwrap();

        let publish = Publish::new(
            false,
            QoS::AtMost,
            false,
            topic_name,
            None,
            vec![b'c'],
        );

        let bytes = publish.to_bytes();

        assert_eq!(bytes, vec![0b0011_0000, 6 as u8, 0x00, 0x03, b'a', b'/', b'b', b'c']);
    }
}