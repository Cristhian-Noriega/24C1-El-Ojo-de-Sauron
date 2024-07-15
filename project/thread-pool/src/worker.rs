use std::{
    sync::{mpsc::Receiver, Arc, Mutex},
    thread,
};

use crate::message::Message;

pub struct Worker {
    pub id: usize,
    pub thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let locked_receiver = match receiver.lock() {
                Ok(receiver) => receiver,
                Err(_) => {
                    println!("Worker {} failed to lock receiver.", id);
                    break;
                }
            };

            let message = match locked_receiver.recv() {
                Ok(message) => message,
                Err(_) => {
                    println!("Worker {} failed to receive message.", id);
                    break;
                }
            };

            drop(locked_receiver);

            match message {
                Message::NewJob(job) => {
                    println!("Worker {} got a job; executing.", id);
                    job();
                }
                Message::Terminate => {
                    println!("Worker {} was told to terminate.", id);
                    break;
                }
            }
        });

        let thread = Some(thread);

        Worker { id, thread }
    }
}
