use std::io::Read;

use crate::{
    errors::error::Error,
    model::package_components::variable_header_components::contents::connect_variable_header_content::ConnectVariableHeaderContent,
};

// TODO: hay que pasar a encoded strings (1.5.3)
pub const CLIENT_ID_LENGTH: usize = 23;
const WILL_TOPIC_LENGTH: usize = 2;
const WILL_MESSAGE_LENGTH: usize = 2;
const USERNAME_LENGTH: usize = 2;
const PASSWORD_LENGTH: usize = 2;

pub struct ConnectPayload {
    client_id: [u8; CLIENT_ID_LENGTH],
    will_topic: Option<[u8; WILL_TOPIC_LENGTH]>,
    will_message: Option<[u8; WILL_MESSAGE_LENGTH]>,
    username: Option<[u8; USERNAME_LENGTH]>,
    password: Option<[u8; PASSWORD_LENGTH]>,
}

impl ConnectPayload {
    pub fn new(
        client_id: [u8; CLIENT_ID_LENGTH],
        will_topic: Option<[u8; WILL_TOPIC_LENGTH]>,
        will_message: Option<[u8; WILL_MESSAGE_LENGTH]>,
        username: Option<[u8; USERNAME_LENGTH]>,
        password: Option<[u8; PASSWORD_LENGTH]>,
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
        let mut client_id = [0; CLIENT_ID_LENGTH];
        stream.read_exact(&mut client_id)?;

        let (will_topic, will_message) = if variable_header_content.has_will() {
            let mut will_topic = [0; WILL_TOPIC_LENGTH];
            stream.read_exact(&mut will_topic)?;

            let mut will_message = [0; WILL_MESSAGE_LENGTH];
            stream.read_exact(&mut will_message)?;

            (Some(will_topic), Some(will_message))
        } else {
            (None, None)
        };

        let username = if variable_header_content.has_username() {
            let mut username = [0; USERNAME_LENGTH];
            stream.read_exact(&mut username)?;

            Some(username)
        } else {
            None
        };

        let password = if variable_header_content.has_password() {
            let mut password = [0; PASSWORD_LENGTH];
            stream.read_exact(&mut password)?;

            Some(password)
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
