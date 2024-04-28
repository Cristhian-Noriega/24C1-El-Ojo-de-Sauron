use crate::{
    errors::error::Error,
    model::package_components::{
        fixed_header::FixedHeader, payload::Payload, variable_header::VariableHeader,
    },
};

use super::package_components::{
    fixed_header::FIXED_HEADER_LENGTH, fixed_header_components::{
        control_packet_type::ControlPacketType,
        flags::Flags,
        qos::QoS,
    }, payload_components::connect_payload::ConnectPayload, variable_header_components::{
        contents::connect_variable_header::ConnectVariableHeader,
        variable_header_content::VariableHeaderContent,
    }
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

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        //no estoy seguro cuantos minimos bytes poner
        if bytes.len() < FIXED_HEADER_LENGTH {
            return Err(Error::new(format!(
                "El package no cumple con el tamaño esperado"
            )));
        }

        //parseo de fixed header

        let fixed_header = FixedHeader::from_bytes(&bytes[0..2])?; //el fixed header ocupa 2 bytes
        let remaining_lenght = fixed_header.get_remaining_length();

        if bytes.len() < FIXED_HEADER_LENGTH + remaining_lenght {
            return Err(Error::new(format!("El package no cumple con el tamaño esperado")));
        }

        //parseo de variable header
        let variable_header_start = FIXED_HEADER_LENGTH;
        let variable_header_end = variable_header_start + remaining_lenght
        let variable_header  = VariableHeader::from_bytes(&bytes[variable_header_start, variable_header_start])?;

        //parseo de payload
        let payload_start  = variable_header_end;
        let payload = bytes[payload_start..].to_vec();

        Ok(Package {
            fixed_header,
            Some(variable_header),
            Some(payload),
        })

    }
}
