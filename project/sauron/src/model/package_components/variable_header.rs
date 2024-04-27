pub struct VariableHeader {
    packet_identifier_msb: u8,
    packet_identifier_lsb: u8,
    content: Vec<u8>,
}

impl VariableHeader {
    pub fn new(packet_identifier_msb: u8, packet_identifier_lsb: u8, content: Vec<u8>) -> Self {
        Self {
            packet_identifier_msb,
            packet_identifier_lsb,
            content,
        }
    }
}
