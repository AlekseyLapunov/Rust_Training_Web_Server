use core::fmt;
use std::{
    thread::{self, JoinHandle},
    sync::{mpsc, Arc, Mutex},
    error::Error
};

#[derive(Debug)]
pub enum PoolCreationError {
    WrongPoolSize,
    WorkerJobTrouble,
}

impl fmt::Display for PoolCreationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let description = match *self {
            PoolCreationError::WrongPoolSize => "wrong thread pool size",
            PoolCreationError::WorkerJobTrouble => "worker did not get a job (problem thread spawning)",
        };
        f.write_str(description)
    }
}

impl Error for PoolCreationError {}

struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}
impl Worker {
    fn build(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Result<Worker, PoolCreationError> {
        let builder = thread::Builder::new();

        let thread = builder.spawn(move || loop { 
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    println!("Worker {id} got a job; executing.");
                    job();
                }

                Err(_) => {
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
        }).map_err(|_| PoolCreationError::WorkerJobTrouble)?;

        Ok(Worker { id, thread: Some(thread) })
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    pub fn build(size: usize) -> Result<ThreadPool, PoolCreationError> {
        if size == 0 {
            return Err(PoolCreationError::WrongPoolSize);
        }

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::build(id, Arc::clone(&receiver))?);
        }

        Ok(ThreadPool { workers, sender: Some(sender) })
    }

    pub fn execute<F>(&self, f: F)
    where 
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

