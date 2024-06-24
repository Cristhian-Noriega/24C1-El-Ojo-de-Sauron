use chrono::Local;
use mqtt::model::packets::{publish::Publish, subscribe::Subscribe, unsubscribe::Unsubscribe};
use std::{
    fs::OpenOptions,
    io::Write,
    sync::mpsc::{self, Sender},
    thread,
};

const LOG_LEVEL_INFO: &str = "INFO";
const LOG_LEVEL_ERROR: &str = "ERROR";

/// Represents a logger that writes to a file
#[derive(Debug, Clone)]
pub struct Logger {
    sender: Sender<String>,
}

impl Logger {
    /// Creates a new logger that writes to the specified file
    pub fn new(log_file_path: &str) -> Self {
        let (sender, receiver) = mpsc::channel();
        let file_path = log_file_path.to_string();
        thread::spawn(move || {
            let mut file = match OpenOptions::new()
                .create(true)
                .append(true)
                .open(&file_path)
            {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("Failed to open log file: {}", e);
                    return;
                }
            };

            for log_entry in receiver {
                if let Err(e) = writeln!(file, "{}", log_entry) {
                    eprintln!("Failed to write to log file: {}", e);
                }
            }
        });

        Logger { sender }
    }

    /// Logs a message with the specified level
    pub fn log(&self, level: &str, message: &str) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let log_entry = format!("[{}] {}: {}", timestamp, level, message);
        self.sender.send(log_entry).unwrap();
    }

    /// Logs an info message
    pub fn info(&self, message: &str) {
        self.log(LOG_LEVEL_INFO, message);
    }

    /// Logs an error message
    pub fn error(&self, message: &str) {
        self.log(LOG_LEVEL_ERROR, message);
    }

    /// Logs a custom message for successful subscription
    pub fn log_successful_subscription(&self, client_id: &[u8], subscribe_packet: &Subscribe) {
        let message = format!(
            "Client {} subscribed to topics {}",
            std::str::from_utf8(client_id).unwrap(),
            subscribe_packet
                .topics()
                .iter()
                .fold(String::new(), |acc, (topic, _)| {
                    acc + std::str::from_utf8(topic.to_string().as_bytes()).unwrap() + ", "
                })
        );
        self.info(message.as_str());
    }

    /// Logs a custom message for successful unsubscription
    pub fn log_successful_unsubscription(
        &self,
        client_id: &[u8],
        unsubscribe_packet: &Unsubscribe,
    ) {
        let message = format!(
            "Client {} unsubscribed to topics {}",
            String::from_utf8_lossy(client_id).to_string(),
            unsubscribe_packet
                .topics()
                .iter()
                .fold(String::new(), |acc, topic| {
                    acc + std::str::from_utf8(topic.to_string().as_bytes()).unwrap() + ", "
                })
        );
        self.info(message.as_str());
    }

    /// Logs a custom message for successful publish
    pub fn log_successful_publish(&self, client_id: &[u8], publish_packet: &Publish) {
        let message = format!(
            "Client {} published message {} to topic {}",
            String::from_utf8_lossy(client_id).to_string(),
            String::from_utf8_lossy(publish_packet.message()).to_string(),
            String::from_utf8_lossy(publish_packet.topic().to_string().as_bytes()).to_string()
        );
        self.info(message.as_str());
    }

    /// Logs a custom message for successful disconnection
    pub fn log_client_does_not_exist(&self, client_id: &[u8]) {
        let message = format!(
            "Client {} does not exist",
            String::from_utf8_lossy(client_id).to_string()
        );
        self.error(message.as_str());
    }

    /// Logs a custom message for successful disconnection
    pub fn log_info_sent_packet(&self, packet_type: &str, client_id: &[u8]) {
        let message = format!(
            "Sent {} packet to client {}",
            packet_type,
            String::from_utf8_lossy(client_id).to_string()
        );
        self.info(message.as_str());
    }

    /// Logs a custom message for successful disconnection
    pub fn log_error_sending_packet(&self, packet_type: &str, client_id: &[u8]) {
        let message = format!(
            "Error sending {} packet to client {}",
            packet_type,
            String::from_utf8_lossy(client_id).to_string()
        );
        self.error(message.as_str());
    }

    /// Logs a custom message for successful disconnection
    pub fn log_error_getting_stream(&self, client_id: &[u8], packet_type: &str) {
        let message = format!(
            "Error getting stream for client {} when sending {} packet",
            String::from_utf8_lossy(client_id).to_string(),
            packet_type
        );
        self.error(message.as_str());
    }

    /// Logs a custom message for successful disconnection
    pub fn log_sent_message(&self, message: String, client_id: String) {
        let message = format!("Sent message: {} to client {}", message, client_id,);
        self.info(message.as_str());
    }

    /// Logs a custom message for successful disconnection
    pub fn log_sending_message_error(&self, message: String, client_id: String) {
        let message = format!("Error sending message: {} to client {}", message, client_id);
        self.error(message.as_str());
    }

    /// Logs a custom message for client registration
    pub fn log_client_registrated(&self, client_id: &[u8]) {
        let message = format!(
            "Client with id {} has been registered successfully",
            String::from_utf8_lossy(client_id).to_string()
        );
        self.info(message.as_str());
    }
}
