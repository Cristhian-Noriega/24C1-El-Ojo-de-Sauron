use crate::{EncodedString, Error, Read};

#[derive(Debug)]
pub struct Login {
    pub username: EncodedString,
    pub password: Option<EncodedString>,
}

impl Login {
    pub fn new(username: EncodedString, password: Option<EncodedString>) -> Login {
        Login { username, password }
    }

    pub fn from_bytes(stream: &mut dyn Read, has_password: bool) -> Result<Login, Error> {
        let username = EncodedString::from_bytes(stream)?;

        let password = if has_password {
            Some(EncodedString::from_bytes(stream)?)
        } else {
            None
        };

        Ok(Login::new(username, password))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend(self.username.to_bytes());

        if let Some(password) = &self.password {
            bytes.extend(password.to_bytes());
        }

        bytes
    }

    pub fn username(&self) -> &EncodedString {
        &self.username
    }

    pub fn password(&self) -> Option<&EncodedString> {
        self.password.as_ref()
    }
}
