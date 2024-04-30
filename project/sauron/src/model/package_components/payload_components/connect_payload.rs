use std::io::Read;

use crate::errors::error::Error;

pub struct ConnectPayload {
    client_id: Vec<u8>,
    will_topic: Option<Vec<u8>>,
    will_message: Option<Vec<u8>>,
    username: Option<Vec<u8>>,
    password: Option<Vec<u8>>,
}

impl ConnectPayload {
    pub fn new(
        client_id: Vec<u8>,
        will_topic: Option<Vec<u8>>,
        will_message: Option<Vec<u8>>,
        username: Option<Vec<u8>>,
        password: Option<Vec<u8>>,
    ) -> Self {
        Self {
            client_id,
            will_topic,
            will_message,
            username,
            password,
        }
    }

    pub fn from_bytes(bytes: &mut dyn Read, remaining_length: usize) -> Result<Self, Error> {
        todo!()
    }

    pub fn into_bytes(self) -> Vec<u8> {
        let mut payload_bytes = vec![];

        payload_bytes.extend(self.client_id);

        if let Some(will_topic) = self.will_topic {
            payload_bytes.extend(will_topic);
        }

        if let Some(will_message) = self.will_message {
            payload_bytes.extend(will_message);
        }

        if let Some(username) = self.username {
            payload_bytes.extend(username);
        }

        if let Some(password) = self.password {
            payload_bytes.extend(password);
        }

        payload_bytes
    }
}

// fn read_string<R: Read>(bytes: &mut R) -> Result<Vec<u8>, Error> {
//     let length = bytes.read_u16::<BigEndian>()?;
//     let mut string = vec![0; length as usize];
//     bytes.read_exact(&mut string)?;

//     Ok(string)
// }
