use std::io::Read;

use crate::{
    errors::error::Error,
    model::{
        encoded_strings::EncodedString,
        package_components::variable_header_components::contents::connect_variable_header_content::ConnectVariableHeaderContent,
    },
};

// TODO: hay que pasar a encoded strings (1.5.3)
pub const CLIENT_ID_LENGTH: usize = 23;
const WILL_TOPIC_LENGTH: usize = 2;
const WILL_MESSAGE_LENGTH: usize = 2;
const USERNAME_LENGTH: usize = 2;
const PASSWORD_LENGTH: usize = 2;

pub struct ConnectPayload {
    client_id: EncodedString,
    will_topic: Option<EncodedString>,
    will_message: Option<EncodedString>,
    username: Option<EncodedString>,
    password: Option<EncodedString>,
}

impl ConnectPayload {
    pub fn new(
        client_id: EncodedString,
        will_topic: Option<EncodedString>,
        will_message: Option<EncodedString>,
        username: Option<EncodedString>,
        password: Option<EncodedString>,
    ) -> Self {
        Self {
            client_id,
            will_topic,
            will_message,
            username,
            password,
        }
    }

    pub fn from_bytes(
        stream: &mut dyn Read,
        remaining_length: usize,
        variable_header_content: &ConnectVariableHeaderContent,
    ) -> Result<Self, Error> {
        let client_id = EncodedString::from_bytes(stream)?;

        let (will_topic, will_message) = if variable_header_content.has_will() {
            (
                Some(EncodedString::from_bytes(stream)?),
                Some(EncodedString::from_bytes(stream)?),
            )
        } else {
            (None, None)
        };

        let username = if variable_header_content.has_username() {
            Some(EncodedString::from_bytes(stream)?)
        } else {
            None
        };

        let password = if variable_header_content.has_password() {
            Some(EncodedString::from_bytes(stream)?)
        } else {
            None
        };

        Ok(Self {
            client_id,
            will_topic,
            will_message,
            username,
            password,
        })
    }

    pub fn into_bytes(&self) -> Vec<u8> {
        todo!()
        // let mut payload_bytes = vec![];

        // payload_bytes.extend(self.client_id);

        // if let Some(will_topic) = self.will_topic {
        //     payload_bytes.extend(will_topic);
        // }

        // if let Some(will_message) = self.will_message {
        //     payload_bytes.extend(will_message);
        // }

        // if let Some(username) = self.username {
        //     payload_bytes.extend(username);
        // }

        // if let Some(password) = self.password {
        //     payload_bytes.extend(password);
        // }

        // payload_bytes
    }
}

// fn read_string<R: Read>(bytes: &mut R) -> Result<Vec<u8>, Error> {
//     let length = bytes.read_u16::<BigEndian>()?;
//     let mut string = vec![0; length as usize];
//     bytes.read_exact(&mut string)?;

//     Ok(string)
// }
