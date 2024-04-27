use crate::model::package_components::fixed_header_components::{
    control_packet_type::ControlPacketType, flags::Flags,
};

pub struct FixedHeader {
    control_packet_type: ControlPacketType,
    flags: Flags,
    remaining_length: u8,
}

impl FixedHeader {
    pub fn new(control_packet_type: ControlPacketType, flags: Flags, remaining_length: u8) -> Self {
        Self {
            control_packet_type,
            flags,
            remaining_length,
        }
    }
}
