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

    pub fn into_bytes(self) -> Vec<u8> {
        let packet_type_bytes = self.control_packet_type.into_u8();
        let flags_bytes = self.flags.into_u8();

        let fixed_header_bytes = vec![packet_type_bytes << 4 | flags_bytes, self.remaining_length];

        fixed_header_bytes
    }

    pub fn from_bytes() -> Self {
        todo!()
    }
}
