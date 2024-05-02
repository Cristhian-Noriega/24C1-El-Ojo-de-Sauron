use std::io::Read;

use crate::{
    errors::error::Error,
    model::package_components::{
        payload_components::encoded_string::EncodedString,
        variable_header_components::contents::variable_header_content_connect::VariableHeaderContentConnect,
    },
};
pub struct PayloadContentConnect {
    client_id: EncodedString,
    will: Option<(EncodedString, EncodedString)>,
    user: Option<(EncodedString, Option<EncodedString>)>,
}

impl PayloadContentConnect {
    pub fn new(
        client_id: EncodedString,
        will: Option<(EncodedString, EncodedString)>,
        user: Option<(EncodedString, Option<EncodedString>)>,
    ) -> Self {
        Self {
            client_id,
            will,
            user,
        }
    }

    pub fn from_bytes(
        stream: &mut dyn Read,
        _remaining_length: usize,
        variable_header_content: &VariableHeaderContentConnect,
    ) -> Result<Self, Error> {
        let client_id = EncodedString::from_bytes(stream)?;

        let will = if variable_header_content.has_will() {
            let will_topic = EncodedString::from_bytes(stream)?;
            let will_message = EncodedString::from_bytes(stream)?;

            Some((will_topic, will_message))
        } else {
            None
        };

        let user = if variable_header_content.has_username() {
            let username = EncodedString::from_bytes(stream)?;

            let password = if variable_header_content.has_password() {
                Some(EncodedString::from_bytes(stream)?)
            } else {
                None
            };

            Some((username, password))
        } else {
            None
        };

        Ok(Self::new(client_id, will, user))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut payload_bytes = vec![];

        payload_bytes.extend(self.client_id.to_bytes());

        if let Some((will_topic, will_message)) = &self.will {
            payload_bytes.extend(will_topic.to_bytes());
            payload_bytes.extend(will_message.to_bytes());
        }

        if let Some((username, password)) = &self.user {
            payload_bytes.extend(username.to_bytes());

            if let Some(password) = password {
                payload_bytes.extend(password.to_bytes());
            }
        }

        payload_bytes
    }

    pub fn get_length(&self) -> usize {
        let mut length = self.client_id.get_length();

        if let Some((will_topic, will_message)) = &self.will {
            length += will_topic.get_length();
            length += will_message.get_length();
        }

        if let Some((username, password)) = &self.user {
            length += username.get_length();

            if let Some(password) = password {
                length += password.get_length();
            }
        }

        length
    }
}
