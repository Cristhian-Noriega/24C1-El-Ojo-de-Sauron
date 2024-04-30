use super::{
    fixed_header_components::control_packet_type::ControlPacketType,
    payload_components::connect_payload::ConnectPayload,
    variable_header_components::variable_header_content::{self, VariableHeaderContent},
};
use crate::errors::error::Error;
use std::io::Read;

pub enum Payload {
    Connect(ConnectPayload),
}

impl Payload {
    pub fn into_bytes(&self) -> Vec<u8> {
        todo!()
    }

    pub fn from_bytes(
        stream: &mut dyn Read,
        control_packet_type: &ControlPacketType,
        remaining_length: usize,
        variable_header_content: &VariableHeaderContent,
    ) -> Result<Self, Error> {
        let variable_header_content = match variable_header_content {
            variable_header_content::VariableHeaderContent::Connect(connect) => connect,
        };

        match control_packet_type {
            ControlPacketType::Connect => {
                let connect =
                    ConnectPayload::from_bytes(stream, remaining_length, variable_header_content)?;
                Ok(Payload::Connect(connect))
            }
        }
    }

    pub fn get_length(&self) -> usize {
        match self {
            Payload::Connect(connect) => connect.get_length(),
        }
    }
}
