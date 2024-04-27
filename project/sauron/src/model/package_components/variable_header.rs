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

    pub fn into_bytes(self) -> Vec<u8> {
        let mut variable_header_bytes =
            vec![self.packet_identifier_msb, self.packet_identifier_lsb];

        variable_header_bytes.extend(self.content);

        variable_header_bytes
    }

    pub fn from_bytes() -> Self {
        todo!();
    }
}
