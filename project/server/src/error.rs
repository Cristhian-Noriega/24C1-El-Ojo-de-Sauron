pub enum ErrorResgistration {
    LockError,
    AuthenticationError,
    IoError(std::io::Error),
    ClientAlreadyRegistered,
    ClientNotFound,
}

pub enum ErrorServer {
    LockError,
}


#[derive(Debug)]
pub struct Error {
    _message: String,
}

impl Error {
    pub fn new(message: String) -> Self {
        Self { _message: message }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self {
            _message: format!("IO error: {}", error),
        }
    }
}
