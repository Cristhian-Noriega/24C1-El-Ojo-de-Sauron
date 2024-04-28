use super::payload_components::connect_payload::ConnectPayload;

pub enum Payload {
    Connect(ConnectPayload),
    // Publish(Publish),
    // Subscribe(Subscribe),
    // SubAck(SubAck),
    // Unsubscribe(Unsubscribe),
}

impl Payload {
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        todo!()
    }

    pub fn into_bytes(self) -> Vec<u8> {
        todo!()
    }

    pub fn get_length(self) -> usize {
        todo!()
    }
}
