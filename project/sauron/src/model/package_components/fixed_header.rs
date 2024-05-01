use std::io::Read;

use crate::{
    errors::error::Error,
    model::package_components::fixed_header_components::control_packet_type::ControlPacketType,
};

use super::fixed_header_components::fixed_header_flags::FixedHeaderFlags;

pub const FIXED_HEADER_LENGTH: usize = 2;

pub struct FixedHeader {
    control_packet_type: ControlPacketType,
    flags: FixedHeaderFlags,
    remaining_length: usize,
}

impl FixedHeader {
    pub fn new(
        control_packet_type: ControlPacketType,
        flags: FixedHeaderFlags,
        remaining_length: usize,
    ) -> Self {
        Self {
            control_packet_type,
            flags,
            remaining_length,
        }
    }

    pub fn from_bytes(stream: &mut dyn Read) -> Result<Self, Error> {
        let mut buffer = [0; FIXED_HEADER_LENGTH];
        stream.read_exact(&mut buffer)?;

        let first_byte = buffer[0];

        let control_packet_type: ControlPacketType = ControlPacketType::from_byte(first_byte)?;
        let flags = FixedHeaderFlags::from_byte(first_byte, &control_packet_type)?;

        stream.read_exact(&mut buffer)?;
        let remaining_length = buffer[1] as usize;

        Ok(Self {
            control_packet_type,
            flags,
            remaining_length,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let packet_type_bytes = self.control_packet_type.to_byte();
        let flags_bytes = self.flags.to_byte();

        let fixed_header_bytes = vec![
            packet_type_bytes << 4 | flags_bytes,
            self.remaining_length as u8,
        ];

        fixed_header_bytes
    }

    pub fn get_remaining_length(&self) -> usize {
        self.remaining_length
    }

    pub fn get_control_packet_type(&self) -> &ControlPacketType {
        &self.control_packet_type
    }
}
