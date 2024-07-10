use std::io;
use mqtt::errors::error::MqttError;

pub enum ClientDroneError {
    InvalidMessage,
    InvalidIncident,
    PoisonedLock,
    MqttError(MqttError),
    ArgumentError(String),
    Io(io::Error), 
    
}