// workers.rs

use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{Arc, Mutex, mpsc},
    thread,
};
use tracing::{error, info, instrument};

/// Job is a one time executable function that takes nothing,
/// returns nothing, and doesn't borrow any static data.
/// Safe to be transferred to another thread.
/// Allocated on heap.
type Job = Box<dyn FnOnce() + Send + 'static>;

/// WorkerPool coordinates the threads running in the background
/// for handling input requests.
pub struct WorkerPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

impl WorkerPool {
    /// new returns a pool of worker threads running in the background.
    #[instrument(skip_all)]
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
            info!(id = id, "starting worker");
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        WorkerPool { workers, sender }
    }

    /// execute accepts an instance of Job type and sends it over a channel.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        // allocate in heap and send over the channel
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}

/// Implementing Drop Train on WorkerPool.
impl Drop for WorkerPool {
    #[instrument(skip_all)]
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
    #[instrument(skip_all)]
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            info!(id = id, "start");

            loop {
                // receive events from the sender channel
                let event = receiver.lock().unwrap().recv();
                match event {
                    Ok(job) => {
                        // execute job and catch unexpected errors
                        if catch_unwind(AssertUnwindSafe(job)).is_err() {
                            error!(id = id, "job panicked, recovering");
                        }
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
