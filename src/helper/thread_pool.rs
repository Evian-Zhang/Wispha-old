use std::thread;
use std::sync::{mpsc, Arc, Mutex};
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter, Debug};

type Result<T> = std::result::Result<T, ThreadPoolError>;

type Job = Box<dyn FnBox + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)();
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

impl ThreadPool {
    pub fn new(size: usize) -> Result<ThreadPool> {
        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for _ in 0..size {
            workers.push(Worker::new(Arc::clone(&receiver))?);
        }

        Ok(ThreadPool {
            workers,
            sender,
        })
    }

    pub fn execute<F>(&self, f: F)
        where
            F: FnOnce() + Send + 'static
    {
        let job = Box::new(f);

        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in &mut self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Result<Worker> {
        let builder = thread::Builder::new();
        let thread = builder.spawn(move || {
            loop {
                if let Ok(message) = receiver.lock().unwrap().recv() {
                    match message {
                        Message::NewJob(job) => {
                            job.call_box();
                        },
                        Message::Terminate => {
                            break;
                        },
                    }
                }
            }
        }).or(Err(ThreadPoolError::NoEnoughThread))?;

        Ok(Worker {
            thread: Some(thread),
        })
    }
}

#[derive(Debug)]
pub enum ThreadPoolError {
    NoEnoughThread,
}

impl Error for ThreadPoolError { }

impl Display for ThreadPoolError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use ThreadPoolError::*;
        let message = match &self {
            NoEnoughThread => {
                format!("No enough thread.")
            }
        };
        write!(f, "{}", message)
    }
}