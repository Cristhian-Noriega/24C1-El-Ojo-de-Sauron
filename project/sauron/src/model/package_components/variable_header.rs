use super::variable_header_components::variable_header_content::VariableHeaderContent;

pub struct VariableHeader {
    packet_identifier_msb: u8,
    packet_identifier_lsb: u8,
    content: VariableHeaderContent,
}

impl VariableHeader {
    pub fn new(
        packet_identifier_msb: u8,
        packet_identifier_lsb: u8,
        content: VariableHeaderContent,
    ) -> Self {
        Self {
            packet_identifier_msb,
            packet_identifier_lsb,
            content,
        }
    }

    pub fn into_bytes(self) -> Vec<u8> {
        let mut variable_header_bytes =
            vec![self.packet_identifier_msb, self.packet_identifier_lsb];

        let content_bytes = self.content.into_bytes();

        variable_header_bytes.extend(content_bytes);

        variable_header_bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let packet_identifier_msb = bytes[0];
        let packet_identifier_lsb = bytes[1];
        //para el content habria que usar el enum ? 
        let content = bytes[2..]

        Ok(VariableHeader::new(
            packet_identifier_msb,
            packet_identifier_lsb,
        ))
    }

    pub fn get_length(self) -> usize {
        self.packet_identifier_msb + self.packet_identifier_lsb + self.content.get_length()
    }
}
