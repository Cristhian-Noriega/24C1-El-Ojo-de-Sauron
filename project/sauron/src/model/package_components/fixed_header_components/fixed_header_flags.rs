use crate::errors::error::Error;

use super::{
    control_packet_type::ControlPacketType,
    flags::fixed_header_flags_publish::FixedHeaderFlagsPublish,
};

pub enum FixedHeaderFlags {
    Reserved,
    Publish(FixedHeaderFlagsPublish),
}

impl FixedHeaderFlags {
    pub fn from_byte(_byte: u8, control_packet_type: &ControlPacketType) -> Result<Self, Error> {
        match control_packet_type {
            ControlPacketType::Connect => Ok(FixedHeaderFlags::Reserved),
        }
    }

    pub fn into_byte(&self) -> u8 {
        match self {
            FixedHeaderFlags::Reserved => 0x00,
            FixedHeaderFlags::Publish(flags) => flags.into_byte(),
        }
    }
}
