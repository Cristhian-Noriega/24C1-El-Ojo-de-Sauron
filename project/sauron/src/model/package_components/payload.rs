use super::{
    fixed_header_components::control_packet_type::ControlPacketType,
    payload_components::connect_payload::ConnectPayload,
};
use crate::errors::error::Error;
use std::io::Read;

pub enum Payload {
    Connect(ConnectPayload),
    // Publish(Publish),
    // Subscribe(Subscribe),
    // SubAck(SubAck),
    // Unsubscribe(Unsubscribe),
}

impl Payload {
    pub fn from_bytes(
        bytes: &mut dyn Read,
        control_packet_type: ControlPacketType,
        remaining_length: usize,
    ) -> Result<Self, Error> {
        match control_packet_type {
            ControlPacketType::Connect => {
                let connect = ConnectPayload::from_bytes(bytes, remaining_length)?;
                Ok(Payload::Connect(connect))
            }
        }
    }

    pub fn get_length(self) -> usize {
        todo!()
    }
}
