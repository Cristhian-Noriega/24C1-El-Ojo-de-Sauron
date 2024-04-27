use crate::model::package_components::{
    fixed_header::FixedHeader,
    fixed_header_components::{
        control_packet_type::ControlPacketType::Connect, flags::Flags, qos::QoS,
    },
    variable_header::VariableHeader,
};
use std::io::{Error, ErrorKind};

// siento que esto no debería estar aquí
const PACKET_IDENTIFIER_MSB: u8 = 0x00;
const PACKET_IDENTIFIER_LSB: u8 = 0x04;
const PROTOCOL_LEVEL: u8 = 0x04;
const CLEAN_SESSION: u8 = 0x02;
const KEEP_ALIVE_MSB: u8 = 10;
const KEEP_ALIVE_LSB: u8 = 10;

pub struct Package {
    fixed_header: FixedHeader,
    variable_header: VariableHeader,
    payload: Vec<u8>,
}

impl Package {
    pub fn build_connect(client_id: &[u8]) -> Result<Self, std::io::Error> {
        let fixed_header = FixedHeader::new(
            Connect,
            Flags::new(false, false, QoS::AtMostOnce),
            (10 + client_id.len()) as u8,
        );

        let variable_header = VariableHeader::new(
            PACKET_IDENTIFIER_MSB,
            PACKET_IDENTIFIER_LSB,
            vec![
                'M' as u8,
                'Q' as u8,
                'T' as u8,
                'T' as u8,
                PROTOCOL_LEVEL,
                CLEAN_SESSION,
                KEEP_ALIVE_MSB,
                KEEP_ALIVE_LSB,
            ],
        );

        let mut payload: Vec<u8> = vec![];
        payload.extend_from_slice(client_id);

        Ok(Package {
            fixed_header,
            variable_header,
            payload,
        })
    }

    pub fn convert(self) -> Vec<u8> {
        todo!()
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
