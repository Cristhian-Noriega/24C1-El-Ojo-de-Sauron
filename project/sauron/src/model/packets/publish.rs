use super::PUBLISH_PACKET_TYPE;
use crate::{Error, FixedHeader, QoS, Read, RemainingLength, TopicName};

const PACKAGE_IDENTIFIER_LENGTH: usize = 2;

#[derive(Debug)]
pub struct Publish {
    dup: bool,
    qos: QoS,
    retain: bool,
    topic: TopicName,
    package_identifier: Option<u16>,
    payload: Vec<u8>,
}

impl Publish {
    pub fn new(
        dup: bool,
        qos: QoS,
        retain: bool,
        topic: TopicName,
        package_identifier: Option<u16>,
        payload: Vec<u8>,
    ) -> Self {
        Self {
            dup,
            qos,
            retain,
            topic,
            package_identifier,
            payload,
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
                let mut package_identifier_buffer = [0; PACKAGE_IDENTIFIER_LENGTH];
                stream.read_exact(&mut package_identifier_buffer)?;

                Some(u16::from_be_bytes(package_identifier_buffer))
            }
        };

        let variable_header_len =
            topic.length() + package_identifier.map_or(0, |_| PACKAGE_IDENTIFIER_LENGTH);

        // Payload

        let payload_len = remaining_length - variable_header_len;

        let mut payload = vec![0; payload_len];
        stream.read_exact(&mut payload)?;

        Ok(Publish::new(
            dup,
            qos,
            retain,
            topic,
            package_identifier,
            payload,
        ))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
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

        let remaining_length_value = variable_header_bytes.len() as u32 + self.payload.len() as u32;
        let remaining_length_bytes = RemainingLength::new(remaining_length_value).to_bytes();
        fixed_header_bytes.extend(remaining_length_bytes);

        let mut packet_bytes = vec![];

        packet_bytes.extend(fixed_header_bytes);
        packet_bytes.extend(variable_header_bytes);
        packet_bytes.extend(&self.payload);

        packet_bytes
    }
}
