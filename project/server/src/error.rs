use mqtt::errors::error::Error as MqttError;
use std::fmt;
use std::io;
use std::sync::mpsc::SendError;
use std::sync::PoisonError;

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
