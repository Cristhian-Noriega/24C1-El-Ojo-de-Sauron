pub struct FixedHeader {
    pub control_packet_type: ControlPacketType,
    pub flags: HeaderFlags,
    pub remaining_length: u8,
}
