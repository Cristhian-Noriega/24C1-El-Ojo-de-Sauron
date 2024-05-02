use super::{
    fixed_header_components::control_packet_type::ControlPacketType,
    payload_components::contents::{
        payload_content_connack::PayloadContentConnack,
        payload_content_connect::PayloadContentConnect,
    },
    variable_header_components::variable_header_content::VariableHeaderContent,
};
use crate::errors::error::Error;
use std::io::Read;

pub enum Payload {
    Connect(PayloadContentConnect),
    Connack(PayloadContentConnack),
}

impl Payload {
    #[allow(clippy::infallible_destructuring_match)]
    pub fn from_bytes(
        stream: &mut dyn Read,
        control_packet_type: &ControlPacketType,
        remaining_length: usize,
        variable_header_content: &VariableHeaderContent,
    ) -> Result<Self, Error> {
        match control_packet_type {
            ControlPacketType::Connect => {
                let variable_header_content = match variable_header_content {
                    VariableHeaderContent::Connect(content) => content,
                    _ => {
                        return Err(Error::new(
                            "Variable header content does not match control packet type"
                                .to_string(),
                        ))
                    }
                };

                let connect = PayloadContentConnect::from_bytes(
                    stream,
                    remaining_length,
                    variable_header_content,
                )?;
                Ok(Payload::Connect(connect))
            }
            ControlPacketType::Connack => {
                let variable_header_content = match variable_header_content {
                    VariableHeaderContent::Connack(content) => content,
                    _ => {
                        return Err(Error::new(
                            "Variable header content does not match control packet type"
                                .to_string(),
                        ))
                    }
                };

                let connack = PayloadContentConnack::from_bytes(
                    stream,
                    remaining_length,
                    variable_header_content,
                )?;
                Ok(Payload::Connack(connack))
            }
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Payload::Connect(connect) => connect.to_bytes(),
            Payload::Connack(connack) => connack.to_bytes(),
        }
    }

    pub fn get_length(&self) -> usize {
        match self {
            Payload::Connect(connect) => connect.get_length(),
            Payload::Connack(connack) => connack.get_length(),
        }
    }
}
