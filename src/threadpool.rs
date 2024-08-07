use core::fmt;
use std::{
    thread::{self, JoinHandle, },
    sync::{mpsc, Arc, Mutex},
};

pub enum PoolCreationError {
    WrongPoolSize,
}

impl fmt::Display for PoolCreationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let description = match *self {
            PoolCreationError::WrongPoolSize => "wrong thread pool size",
        };
        f.write_str(description)
    }
}

struct Worker {
    id: usize,
    thread: JoinHandle<()>,
}
impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop { 
            let job = receiver.lock()
                .expect(format!("Something went wrong: Worker {id} did not acquire a mutex").as_str()) // mutex can be poisoned so this line is vital
                .recv() // receiving the job from the channel and it's blocking
                .expect(format!("Something went wrong: Worker {id} did not get a job").as_str()); 

            println!("Worker {id} got a job! Executing;");

            job(); 
        }); // could be using builder for safety

        Worker { id, thread }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
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
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        Ok(ThreadPool { workers, sender })
    }

    pub fn execute<F>(&self, f: F)
    where 
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(job).unwrap();
    }
}