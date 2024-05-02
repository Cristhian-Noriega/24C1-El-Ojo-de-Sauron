use std::io::Read;

use crate::{
    errors::error::Error,
    model::package_components::fixed_header_components::control_packet_type::ControlPacketType,
};

use super::contents::{
    variable_header_content_connack::VariableHeaderContentConnack,
    variable_header_content_connect::VariableHeaderContentConnect,
};

pub enum VariableHeaderContent {
    Connect(VariableHeaderContentConnect),
    Connack(VariableHeaderContentConnack),
}

impl VariableHeaderContent {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            VariableHeaderContent::Connect(connect) => connect.to_bytes(),
            VariableHeaderContent::Connack(connack) => connack.to_bytes(),
        }
    }

    pub fn from_bytes(
        stream: &mut dyn Read,
        control_packet_type: &ControlPacketType,
    ) -> Result<Self, Error> {
        match control_packet_type {
            ControlPacketType::Connect => {
                let connect = VariableHeaderContentConnect::from_bytes(stream)?;
                Ok(VariableHeaderContent::Connect(connect))
            }
            ControlPacketType::Connack => {
                let connack = VariableHeaderContentConnack::from_bytes(stream)?;
                Ok(VariableHeaderContent::Connack(connack))
            }
        }
    }

    pub fn get_length(&self) -> usize {
        match self {
            VariableHeaderContent::Connect(connect) => connect.get_length(),
            VariableHeaderContent::Connack(connack) => connack.get_length(),
        }
    }
}
