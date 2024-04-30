use std::io::Read;

use crate::{
    errors::error::Error,
    model::package_components::fixed_header_components::control_packet_type::ControlPacketType,
};

use super::contents::connect_variable_header::ConnectVariableHeader;

pub enum VariableHeaderContent {
    Connect(ConnectVariableHeader),
}

impl VariableHeaderContent {
    pub fn into_bytes(self) -> Vec<u8> {
        match self {
            VariableHeaderContent::Connect(connect) => connect.into_bytes(),
        }
    }

    pub fn from_bytes(
        stream: &mut dyn Read,
        control_packet_type: ControlPacketType,
    ) -> Result<Self, Error> {
        match control_packet_type {
            ControlPacketType::Connect => {
                let connect = ConnectVariableHeader::from_bytes(stream)?;
                Ok(VariableHeaderContent::Connect(connect))
            }
        }
    }

    pub fn get_length(self) -> usize {
        match self {
            VariableHeaderContent::Connect(connect) => connect.get_length(),
        }
    }
}
