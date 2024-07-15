use std::fmt;
use std::io;
use std::string::FromUtf8Error;
use std::sync::mpsc::SendError;
use std::sync::PoisonError;

use mqtt::errors::error::MqttError;

pub type ServerResult<T> = Result<T, ServerError>;

#[derive(Debug)]
pub enum ServerError {
    Io(io::Error),
    Mqtt(MqttError),
    ArgumentError(String),
    ClientConnection(String),
    UnsupportedPacket,
    ChannelSend(String),
    PoisonedLock,
    Utf8Error(FromUtf8Error),
    NoLoginProvided,
    NoPasswordProvided,
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerError::Io(err) => write!(f, "I/O error: {}", err),
            ServerError::Mqtt(err) => write!(f, "MQTT error: {:?}", err),
            ServerError::ArgumentError(msg) => write!(f, "Argument error: {}", msg),
            ServerError::ClientConnection(msg) => write!(f, "Client connection error: {}", msg),
            ServerError::UnsupportedPacket => write!(f, "Unsupported packet error"),
            ServerError::ChannelSend(msg) => write!(f, "Channel send error: {}", msg),
            ServerError::PoisonedLock => write!(f, "Poisoned lock error"),
            ServerError::Utf8Error(err) => write!(f, "UTF-8 error: {}", err),
            ServerError::NoLoginProvided => write!(f, "No login provided"),
            ServerError::NoPasswordProvided => write!(f, "No password provided"),
        }
    }
}

impl From<io::Error> for ServerError {
    fn from(err: io::Error) -> Self {
        ServerError::Io(err)
    }
}

impl From<MqttError> for ServerError {
    fn from(err: MqttError) -> Self {
        ServerError::Mqtt(err)
    }
}

impl<T> From<SendError<T>> for ServerError {
    fn from(err: SendError<T>) -> Self {
        ServerError::ChannelSend(err.to_string())
    }
}

impl<T> From<PoisonError<T>> for ServerError {
    fn from(_: PoisonError<T>) -> Self {
        ServerError::PoisonedLock
    }
}

impl From<FromUtf8Error> for ServerError {
    fn from(err: FromUtf8Error) -> Self {
        ServerError::Utf8Error(err)
    }
}
