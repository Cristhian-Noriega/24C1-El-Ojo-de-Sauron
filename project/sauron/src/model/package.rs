use std::io::Read;

use crate::{
    errors::error::Error,
    model::package_components::{
        fixed_header::FixedHeader, payload::Payload, variable_header::VariableHeader,
    },
};

use super::package_components::{
    fixed_header_components::{control_packet_type::ControlPacketType, qos::QoS},
    payload_components::connect_payload::ConnectPayload,
    variable_header_components::{
        contents::connect_variable_header::ConnectVariableHeader,
        variable_header_content::VariableHeaderContent,
    },
};

pub struct Package {
    fixed_header: FixedHeader,
    variable_header: Option<VariableHeader>,
    payload: Option<Payload>,
}

impl Package {
    pub fn build_connect(
        client_id: Vec<u8>,
        variable_header_content: ConnectVariableHeader,
        payload_content: ConnectPayload,
    ) -> Result<Self, Error> {
        let payload = Payload::Connect(payload_content);
        let variable_header = VariableHeader::new(
            0,
            0,
            VariableHeaderContent::Connect(variable_header_content),
        );

        let remaining_len = payload.get_length() + variable_header.get_length();
        let flags = Flags::new(false, false, QoS::AtMostOnce);

        let fixed_header = FixedHeader::new(ControlPacketType::Connect, flags, remaining_len);

        let package = Package {
            fixed_header,
            variable_header: Some(variable_header),
            payload: Some(payload),
        };

        Ok(package)
    }

    pub fn into_bytes(self) -> Vec<u8> {
        let mut package_bytes: Vec<u8> = vec![];

        let fixed_header_bytes = self.fixed_header.into_bytes();
        package_bytes.extend(fixed_header_bytes);

        if let Some(variable_header) = self.variable_header {
            let variable_header_bytes = variable_header.into_bytes();
            package_bytes.extend(variable_header_bytes);
        }

        if let Some(payload) = self.payload {
            let payload_bytes = payload.into_bytes();
            package_bytes.extend(payload_bytes);
        }

        package_bytes
    }

    pub fn from_bytes(stream: &mut dyn Read) -> Result<Self, Error> {
        let fixed_header = FixedHeader::from_bytes(stream)?;
        let remaining_lenght = fixed_header.get_remaining_length();
        let control_packet_type = fixed_header.get_control_packet_type();

        let variable_header = VariableHeader::from_bytes(stream, control_packet_type)?;
        remaining_lenght -= variable_header.get_length();

        let payload = Payload::from_bytes(stream, control_packet_type, remaining_lenght)?;

        Ok(Package {
            fixed_header,
            variable_header: Some(variable_header),
            payload: Some(payload),
        })
    }
}
