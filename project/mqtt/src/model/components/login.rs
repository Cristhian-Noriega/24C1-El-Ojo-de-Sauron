use crate::{EncodedString, Error, Read};

#[derive(Debug, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_to_bytes() {
        let username = EncodedString::from_string(&"username".to_string());
        let password = Some(EncodedString::from_string(&"password".to_string()));
        let login = Login::new(username, password);

        let bytes = login.to_bytes();

        assert_eq!(bytes, vec![0x00, 8, b'u', b's', b'e', b'r', b'n', b'a', b'm', b'e', 0x00, 8, b'p', b'a', b's', b's', b'w', b'o', b'r', b'd']);
    }

    #[test]
    fn test_login_from_bytes() {
        let mut stream = &b"\x00\x08username\x00\x08password"[..];
        let login = Login::from_bytes(&mut stream, true).unwrap();

        assert_eq!(login.username(), &EncodedString::from_string(&"username".to_string()));
        assert_eq!(login.password(), Some(&EncodedString::from_string(&"password".to_string())));
    }
}