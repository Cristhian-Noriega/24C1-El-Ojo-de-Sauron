use std::sync::{mpsc, Arc, Mutex};

use crate::message::Message;
use crate::worker::Worker;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        match self.sender.send(Message::NewJob(job)) {
            Ok(_) => {}
            Err(_) => {
                println!("Failed to send job to worker.");
            }
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in &self.workers {
            match self.sender.send(Message::Terminate) {
                Ok(_) => {}
                Err(_) => {
                    println!("Failed to send terminate message to worker.");
                }
            }
        }

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                println!("Shutting down worker {}", worker.id);

                match thread.join() {
                    Ok(_) => {}
                    Err(_) => {
                        println!("Failed to join worker {}", worker.id);
                    }
                }
            }
        }
    }
}
