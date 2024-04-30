use super::payload_components::connect_payload::ConnectPayload;
use crate::errors::error::Error;
use std::convert::TryFrom;

pub enum Payload {
    Connect(ConnectPayload),
    // Publish(Publish),
    // Subscribe(Subscribe),
    // SubAck(SubAck),
    // Unsubscribe(Unsubscribe),
}

impl Payload {
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, Error> {
        if bytes.is_empty() {
            return Err(Error::InvalidPayload);
        }

        match bytes[0] {
            // Connect payload
            0x10 => {
                let connect_payload = ConnectPayload::try_from(bytes)?;
                Ok(Payload::Connect(connect_payload))
            }
            // Publish payload
            0x30 => {}

            _ => Err(Error::InvalidPayload),
        }
    }

    pub fn get_length(self) -> usize {
        todo!()
    }
}
