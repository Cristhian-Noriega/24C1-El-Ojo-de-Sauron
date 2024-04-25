pub struct VariableHeader {
    pub packet_identifier_msb: u8,
    pub packet_identifier_lsb: u8,
    pub content: Vec<u8>,
}
