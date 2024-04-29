use crate::model::package_components::variable_header_components::contents::connect_variable_header::ConnectVariableHeader;

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

    //pruebo pasandole cualquier tipo que implemente Read para mayor flexibilidad
    pub fn from_bytes(bytes: &mut R, flags: ConnectVariableHeader) -> Self {
        let client_id = read_string(bytes)?;
        let will_topic = if flags.will_flag {
            Some(read_string(bytes)?)
        } else {
            None
        };
        let will_message = if flags.will_flag {
            Some(read_string(bytes)?)
        } else {
            None
        };
        let username = if flags.username_flag {
            Some(read_string(bytes)?)
        } else {
            None
        };
        let password = if flags.password_flag {
            Some(read_string(bytes)?)
        } else {
            None
        };

        Ok(ConnectPayload {
            client_id,
            will_topic,
            will_message,
            username,
            password,
        })
    }

    pub fn into_bytes(self) -> Vec<u8> {
        let mut payload_bytes = vec![self.client_id];

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


fn read_string<R: Read>(bytes: &mut R) -> Result<Vec<u8>, Error> {
    let length = bytes.read_u16::<BigEndian>()?;
    let mut string = vec![0; length as usize];
    bytes.read_exact(&mut string)?;

    Ok(string)
}
