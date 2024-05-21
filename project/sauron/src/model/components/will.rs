use crate::{EncodedString, Error, QoS, Read, TopicName};

#[derive(Debug)]
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

    pub fn from_bytes(stream: &mut dyn Read, qos: QoS, retain: bool) -> Result<Will, Error> {
        let topic = TopicName::from_bytes(stream)?;
        let message = EncodedString::from_bytes(stream)?;

        Ok(Will::new(qos, retain, topic, message))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend(self.topic.to_bytes());
        bytes.extend(self.message.to_bytes());

        bytes
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

    pub fn message(&self) -> &EncodedString {
        &self.message
    }
}
