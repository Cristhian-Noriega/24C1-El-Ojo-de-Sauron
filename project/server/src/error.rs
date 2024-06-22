use std::io;
use std::fmt;
use std::sync::mpsc::SendError;
use mqtt::errors::error::Error as MqttError;

pub type ServerResult<T> = Result<T, ServerError>;

#[derive(Debug)]
pub enum ServerError {
    Io(io::Error),
    Mqtt(MqttError),
    Config(String),
    ArgumentError(String),
    ClientConnection(String),
    UnsupportedPacket(String),
    ChannelSend(String),
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerError::Io(err) => write!(f, "I/O error: {}", err),
            ServerError::Mqtt(err) => write!(f, "MQTT error: {:?}", err),
            ServerError::Config(msg) => write!(f, "Configuration error: {}", msg),
            ServerError::ArgumentError(msg) => write!(f, "Argument error: {}", msg),
            ServerError::ClientConnection(msg) => write!(f, "Client connection error: {}", msg),
            ServerError::UnsupportedPacket(msg) => write!(f, "Unsupported packet error: {}", msg),
            ServerError::ChannelSend(msg) => write!(f, "Channel send error: {}", msg),
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

impl ServerError {
    pub fn argument_error<T: Into<String>>(msg: T) -> Self {
        ServerError::ArgumentError(msg.into())
    }

    pub fn unsupported_packet<T: Into<String>>(msg: T) -> Self {
        ServerError::UnsupportedPacket(msg.into())
    }
}