use chrono::Local;
use std::{
    fs::OpenOptions,
    io::Write,
    sync::mpsc::{self, Sender},
    thread,
};

#[derive(Debug, Clone)]
pub struct Logger {
    sender: Sender<String>,
}

impl Logger {
    pub fn new(log_file_path: &str) -> Self {
        let (sender, receiver) = mpsc::channel();
        let file_path = log_file_path.to_string();
        thread::spawn(move || {
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&file_path)
                .unwrap();

            for log_entry in receiver {
                if let Err(e) = writeln!(file, "{}", log_entry) {
                    eprintln!("Failed to write to log file: {}", e);
                }
            }
        });
        Logger { sender }
    }

    pub fn log(&self, level: &str, message: &str) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let log_entry = format!("[{}] {}: {}", timestamp, level, message);
        self.sender.send(log_entry).unwrap();
    }

    pub fn info(&self, message: &str) {
        self.log("INFO", message);
    }

    pub fn error(&self, message: &str) {
        self.log("ERROR", message);
    }
}
