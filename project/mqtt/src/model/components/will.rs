use crate::{EncodedString, Error, QoS, Read, TopicName};

/// Representa un mensaje que se publicará en caso de que el cliente se desconecte inesperadamente.
#[derive(Debug, PartialEq)]
pub struct Will {
    qos: QoS,
    retain: bool,
    topic: TopicName,
    message: EncodedString,
}

impl Will {
    pub fn new(qos: QoS, retain: bool, topic: TopicName, message: EncodedString) -> Will {
        Will {
            qos,
            retain,
            topic,
            message,
        }
    }

    /// Convierte un stream de bytes en un Will.
    pub fn from_bytes(stream: &mut dyn Read, qos: QoS, retain: bool) -> Result<Will, Error> {
        let topic = TopicName::from_bytes(stream)?;
        let message = EncodedString::from_bytes(stream)?;

        Ok(Will::new(qos, retain, topic, message))
    }

    /// Convierte el Will en un vector de bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend(self.topic.to_bytes());
        bytes.extend(self.message.to_bytes());

        bytes
    }

    /// Devuelve el QoS del Will.
    pub fn qos(&self) -> &QoS {
        &self.qos
    }

    /// Devuelve si el mensaje del Will se retiene.
    pub fn retain(&self) -> bool {
        self.retain
    }

    /// Devuelve el tópico del Will.
    pub fn topic(&self) -> &TopicName {
        &self.topic
    }

    /// Devuelve el mensaje del Will.
    pub fn message(&self) -> &EncodedString {
        &self.message
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[allow(dead_code)]
    fn from_slice(bytes: &[u8]) -> impl Read {
        let encoded_string = EncodedString::new(bytes.to_vec());
        Cursor::new(encoded_string.to_bytes())
    }

    #[test]
    fn test_will_to_bytes() {
        let qos = QoS::AtLeast;
        let retain = true;
        let topic_bytes = &mut from_slice(b"home/livingroom");
        let topic = TopicName::from_bytes(topic_bytes).unwrap();
        let message = EncodedString::from_string(&"message".to_string());
        let will = Will::new(qos, retain, topic, message);

        let bytes = will.to_bytes();

        assert_eq!(bytes, vec![0, 15, b'h', b'o', b'm', b'e', b'/', b'l', b'i', b'v', b'i', b'n', b'g', b'r', b'o', b'o', b'm', 0, 7, b'm', b'e', b's', b's', b'a', b'g', b'e']);
    }
}
