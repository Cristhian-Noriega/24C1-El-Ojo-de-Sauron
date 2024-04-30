use crate::errors::error::Error;

use super::{
    control_packet_type::ControlPacketType,
    flags::fixed_header_flags_connect::FixedHeaderFlagsConnect,
};

pub enum FixedHeaderFlags {
    Connect(FixedHeaderFlagsConnect),
}

impl FixedHeaderFlags {
    pub fn from_byte(byte: u8, control_packet_type: ControlPacketType) -> Result<Self, Error> {
        match control_packet_type {
            ControlPacketType::Connect => {
                let fixed_header_flags_connect = FixedHeaderFlagsConnect::from_byte(byte)?;
                Ok(FixedHeaderFlags::Connect(fixed_header_flags_connect))
            }
        }
    }

    pub fn into_byte(&self) -> u8 {
        match self {
            FixedHeaderFlags::Connect(fixed_header_flags_connect) => {
                fixed_header_flags_connect.into_byte()
            }
        }
    }
}
