use std::fmt;

pub type MqttResult<T> = Result<T, MqttError>;

#[derive(Debug)]
pub enum MqttError {
    InvalidQoSLevel,
    InvalidReserverdFlag,
    InvalidTopicName,
    InvalidProtocolName,
    InvalidProtocolLevel,
    InvalidRemainingLength,
    InvalidWillQoS,
    InvalidWillRetainFlag,
    InvalidPasswordFlag,
    InvalidFixedHeaderFlags,
    NoTopicsSpecified,
    InvalidPacketType(String),
    ErrorDecryption(String),
    InvalidWildcard(String),
    InvalidReturnCode(String),
    IoError(std::io::Error),
}

impl fmt::Display for MqttError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MqttError::InvalidQoSLevel => write!(f, "Invalid QoS level"),
            MqttError::InvalidReserverdFlag => write!(f, "Invalid reserved flag"),
            MqttError::InvalidTopicName => write!(f, "Invalid topic name"),
            MqttError::InvalidProtocolName => write!(f, "Invalid protocol name"),
            MqttError::InvalidProtocolLevel => write!(f, "Invalid protocol level"),
            MqttError::InvalidRemainingLength => write!(f, "Invalid remaining Length"),
            MqttError::InvalidWillQoS => write!(f, "Invalid Will QoS"),
            MqttError::InvalidWillRetainFlag => write!(f, "Invalid Will retain flag"),
            MqttError::InvalidPasswordFlag => write!(f, "Invalid password"),
            MqttError::InvalidFixedHeaderFlags => write!(f, "Invalid fixed header flags"),
            MqttError::NoTopicsSpecified => write!(f, "No topics specified in the payload"),
            MqttError::InvalidPacketType(msg) => write!(f, "Invalid packet type: {}", msg),
            MqttError::ErrorDecryption(msg) => write!(f, "Error decrypting content: {}", msg),
            MqttError::InvalidWildcard(msg) => write!(f, "Invalid Wildcard: {}", msg),
            MqttError::InvalidReturnCode(msg) => write!(f, "Invalid Return Code: {}", msg),
            MqttError::IoError(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl From<std::io::Error> for MqttError {
    fn from(error: std::io::Error) -> Self {
        MqttError::IoError(error)
    }
}