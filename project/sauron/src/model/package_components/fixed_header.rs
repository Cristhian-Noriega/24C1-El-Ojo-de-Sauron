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

    pub fn into_bytes(self) -> Vec<u8> {
        let packet_type_bytes = self.control_packet_type.into_u8();
        let flags_bytes = self.flags.into_u8();

        let fixed_header_bytes = vec![
            packet_type_bytes << 4 | flags_bytes,
            self.remaining_length as u8,
        ];

        fixed_header_bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let first_byte = bytes[0];

        let control_packet_type = ControlPacketType::from_u8(first_byte)?;
        let flags = FixedHeaderFlags::from_u8(first_byte)?;

        let remaining_length = bytes[1] as usize;

        Ok(Self {
            control_packet_type,
            flags,
            remaining_length,
        })
    }

    pub fn get_remaining_leght(&self) -> usize {
        self.remaining_length
    }
}
