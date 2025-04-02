use std::{
    sync::{Arc, Mutex, mpsc},
    thread,
};

/// Stores a bounded pool of threads which may handle various `Job`s.
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

/// A unit of work which is to be completed by a thread.
type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// `size` is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> Self {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)))
        }

        Self {
            workers,
            sender: Some(sender),
        }
    }

    /// Executes a unit of work on a thread in this `ThreadPool`.
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
        for worker in &mut self.workers.drain(..) {
            println!("Shutting down worker {}", worker.id);
            worker.thread.join().unwrap();
        }
    }
}

/// Wrapper for a thread in a `ThreadPool` which shall handle `Job`s.
struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    /// Creates a new `Worker` which will handle `Job`s received via the
    /// `receiver` until that channel is closed.
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
        let thread = thread::spawn(move || {
            loop {
                let msg = receiver.lock().unwrap().recv();
                match msg {
                    Ok(job) => {
                        println!("Worker {id} got a job. Executing...");
                        job()
                    }
                    Err(_) => {
                        println!("Worker {id} disconnected. Shutting down...");
                        break;
                    }
                }
            }
        });

        Self { id, thread }
    }
}
