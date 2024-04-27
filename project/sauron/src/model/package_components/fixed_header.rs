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

    pub fn convert_to_binary(self) -> Vec<u8> {
        // Convierte la struct Package en un Vector de binarios

        let packet_type_value = match self.control_packet_type {
            ControlPacketType::Connect => 0x01,
            ControlPacketType::ConnAck => 0x02,
            ControlPacketType::Publish => 0x03,
            ControlPacketType::PubAck => 0x04,
            ControlPacketType::PubRec => 0x05,
            ControlPacketType::PubRel => 0x06,
            ControlPacketType::PubComp => 0x07,
            ControlPacketType::Subscribe => 0x08,
            ControlPacketType::SubAck => 0x09,
            ControlPacketType::Unsubscribe => 0x0A,
            ControlPacketType::UnsubAck => 0x0B,
            ControlPacketType::PingReq => 0x0C,
            ControlPacketType::PingResp => 0x0D,
            ControlPacketType::Disconnect => 0x0E,
        };

        let fixed_header_binary = vec![
            packet_type_value, // + flags
            0 as u8,
        ];

        return fixed_header_binary
    }
}
