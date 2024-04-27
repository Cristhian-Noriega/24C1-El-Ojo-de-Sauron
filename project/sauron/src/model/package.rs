use crate::{
    errors::error::Error,
    model::package_components::{
        fixed_header::FixedHeader,
        fixed_header_components::{
            control_packet_type::ControlPacketType::Connect, flags::Flags, qos::QoS,
        },
        variable_header::VariableHeader,
    },
};

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
