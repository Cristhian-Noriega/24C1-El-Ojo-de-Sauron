use std::io::Read;

use crate::{
    errors::error::Error,
    model::package_components::{
        fixed_header::FixedHeader, payload::Payload, variable_header::VariableHeader,
    },
};

pub struct Package {
    fixed_header: FixedHeader,
    variable_header: Option<VariableHeader>,
    payload: Option<Payload>,
}

impl Package {
    pub fn new(
        fixed_header: FixedHeader,
        variable_header: Option<VariableHeader>,
        payload: Option<Payload>,
    ) -> Self {
        Self {
            fixed_header,
            variable_header,
            payload,
        }
    }

    pub fn from_bytes(stream: &mut dyn Read) -> Result<Self, Error> {
        let fixed_header = FixedHeader::from_bytes(stream)?;
        let mut remaining_length = fixed_header.get_remaining_length();
        let control_packet_type = fixed_header.get_control_packet_type();

        let variable_header = VariableHeader::from_bytes(stream, control_packet_type)?;
        remaining_length -= variable_header.get_length();
        let variable_header_content = variable_header.get_content();

        let payload = Payload::from_bytes(
            stream,
            control_packet_type,
            remaining_length,
            variable_header_content,
        )?;

        Ok(Package {
            fixed_header,
            variable_header: Some(variable_header),
            payload: Some(payload),
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut package_bytes: Vec<u8> = vec![];

        if let Some(payload) = &self.payload {
            let payload_bytes = payload.to_bytes();
            package_bytes.extend(payload_bytes);
        }

        if let Some(variable_header) = &self.variable_header {
            let variable_header_bytes = variable_header.to_bytes();
            package_bytes.extend(variable_header_bytes);
        }

        let fixed_header_bytes = self.fixed_header.to_bytes();
        package_bytes.extend(fixed_header_bytes);

        package_bytes
    }
}
