// workers.rs

use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use tracing::info;

/// Job is a executable function that returns at once wrapped inside a box.
type Job = Box<dyn FnOnce() + Send + 'static>;

/// WorkerPool coordinates the threads running in the background
/// for handling input requests.
pub struct WorkerPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

impl WorkerPool {
    /// new returns a pool of worker threads running in the background.
    pub fn new(size: usize) -> WorkerPool {
        // must have minimum one handler
        assert!(size > 0);

        // create communication channel
        let (sender, receiver) = mpsc::channel();

        // receiver will be shared with all threads, so it must be a mutex shared variable
        let receiver = Arc::new(Mutex::new(receiver));

        // create and start worker threads
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            // spawn worker threads
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        WorkerPool { workers, sender }
    }

    /// execute accepts an instance of Job type and sends it over a channel.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}

/// Implementing Drop Train on WorkerPool.
impl Drop for WorkerPool {
    fn drop(&mut self) {
        for worker in self.workers.drain(..) {
            info!(id = worker.id, "shutting down worker");
            worker.thread.join().unwrap();
        }
    }
}

/// Worker is a single thread running in the background.
struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    /// new starts a worker thread in background that waits on a channel to pick
    /// Job instances.
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                let message = receiver.lock().unwrap().recv();

                match message {
                    Ok(job) => {
                        job();
                    }
                    Err(_) => {
                        info!(id = id, "shutdown");
                        break;
                    }
                }
            }
        });

        Worker { id, thread }
    }
}
