use std::io::Read;

use crate::errors::error::Error;

use super::{
    fixed_header_components::control_packet_type::ControlPacketType,
    variable_header_components::variable_header_content::VariableHeaderContent,
};

const PACKAGE_IDENTIFIER_LENGTH: usize = 2;

pub struct VariableHeader {
    packet_identifier: u16,
    content: VariableHeaderContent,
}

impl VariableHeader {
    pub fn new(packet_identifier: u16, content: VariableHeaderContent) -> Self {
        Self {
            packet_identifier,
            content,
        }
    }

    pub fn from_bytes(
        stream: &mut dyn Read,
        control_packet_type: &ControlPacketType,
    ) -> Result<Self, Error> {
        let buffer = &mut [0; PACKAGE_IDENTIFIER_LENGTH];
        stream.read_exact(buffer)?;

        let packet_identifier = u16::from_be_bytes(*buffer);

        let content = VariableHeaderContent::from_bytes(stream, control_packet_type)?;

        Ok(VariableHeader::new(packet_identifier, content))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut variable_header_bytes = vec![];

        let packet_identifier_bytes = self.packet_identifier.to_be_bytes();
        variable_header_bytes.extend(packet_identifier_bytes);

        let content_bytes = self.content.to_bytes();
        variable_header_bytes.extend(content_bytes);

        variable_header_bytes
    }

    pub fn get_length(&self) -> usize {
        PACKAGE_IDENTIFIER_LENGTH + self.content.get_length()
    }

    pub fn get_content(&self) -> &VariableHeaderContent {
        &self.content
    }
}
