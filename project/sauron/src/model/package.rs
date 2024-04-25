use crate::FixedHeader;
use crate::VariableHeader;
use std::io::{Error, ErrorKind};

pub struct Package {
    pub fixed_header: FixedHeader,
    pub variable_header: VariableHeader,
    pub payload: Vec<u8>,
}

impl Package{
    pub fn build_connect(client_id: &[u8]) -> Result<Self, std::io::Error> {
        let fixed_header = FixedHeader {
            control_packet_type: Connect,
            flags: Flags {
                retain: false,
                dup: false,
                qos: Qos::AtMostOnce
            },
            remaining_length: (10 + client_id.len()) as u8,
        };

        let variable_header = VariableHeader {
            packet_identifier_msb: 0x0,
            packet_identifier_lsb: 0x04,
            content: vec![
                'M' as u8,
                'Q' as u8,
                'T' as u8,
                'T' as u8,
                0x04, // Protocol level (4 == 3.1.1)
                0x02, // Flags: Clean session
                0x0, // Keep Alive MSB
                10, // Keep Alive LSB
            ]
        };

        let payload: Vec<u8> = vec![];
        payload.extend_from_slice(client_id);
    

        Ok(Package{fixed_header, variable_header, payload})
    }

    pub fn convert(self) -> Vec<u8>{

        // Convierte la struct Package en un Vector de binarios que se pueda transmitir

        // Ejemplo
        // let mut connect_message = vec![
        //     // Fixed Header
        //     0x10, // 1- CONNECT header (DONE)
        //     10,  // 2- Remaining length (DONE)
    
        //     // Variable Header
        //     0x0, // 1- Length MSB (0)
        //     0x04, // 2- Length LSB (4)
        //     'M' as u8, // 3
        //     'Q' as u8, // 4
        //     'T' as u8, // 5
        //     'T' as u8, // 6
        //     0x04, // 7- Protocol level (4 == 3.1.1)
        //     0x02, // 8- Flags: Clean session
        //     0x0, // 9- Keep Alive MSB
        //     10, // 10- Keep Alive LSB
        // ];

        // return connect_message
    }

}
