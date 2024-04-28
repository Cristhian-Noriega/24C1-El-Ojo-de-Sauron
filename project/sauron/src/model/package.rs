use crate::{
    errors::error::Error,
    model::package_components::{
        fixed_header::FixedHeader,
        fixed_header_components::{
            control_packet_type::ControlPacketType::Connect, flags::Flags, qos::QoS,
        },
        variable_header::VariableHeader,
    },
    model::constants::{
        PACKET_IDENTIFIER_MSB, PACKET_IDENTIFIER_LSB, PROTOCOL_LEVEL, CLEAN_SESSION, KEEP_ALIVE_MSB, KEEP_ALIVE_LSB,
    },
};

pub struct Package {
    fixed_header: FixedHeader,
    variable_header: VariableHeader,
    payload: Vec<u8>,
}

impl Package {
    // TODO: Porque client_id ?? no me parece un gran nombre
    pub fn build_connect(client_id: &[u8]) -> Result<Self, Error> {
        let fixed_header = FixedHeader::new(
            Connect,
            Flags::new(false, false, QoS::AtMostOnce),
            (10 + client_id.len()) as u8,
        );

        let variable_header = VariableHeader::new(
            PACKET_IDENTIFIER_MSB,
            PACKET_IDENTIFIER_LSB,
            vec![
                b'M',
                b'Q',
                b'T',
                b'T',
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

    pub fn into_bytes(self) -> Vec<u8> {
        let mut package_bytes: Vec<u8> = vec![];

        let fixed_header_bytes = self.fixed_header.into_bytes();
        let variable_header_bytes = self.variable_header.into_bytes();

        package_bytes.extend(fixed_header_bytes);
        package_bytes.extend(variable_header_bytes);
        package_bytes.extend(self.payload);

        //let remaining_length = variable_header_bytes.len() + payload.len();
        //message[1] = remaining_length as u8;

        package_bytes
    }

    pub fn from_bytes() -> Self {
        todo!()
    }
}
