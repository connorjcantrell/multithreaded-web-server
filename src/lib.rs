use std::thread;
use std::sync::mpsc;
use std::sync::Mutex;
use std::sync::Arc;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        // Create new channel and destructure sender and receiver
        let (sender, receiver) = mpsc::channel();

        // Create one receiving end
        let receiver = Arc::new(Mutex::new(receiver));

        // Set thread capacity, to avoid overloading the server
        let mut workers = Vec::with_capacity(size);

        // Push `workers` onto empty vector
        for id in 0..size {
            // Every `Worker` atomically references `receiver`, but also has it's own unique `id`
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            // Closures are stored in sender, waiting to be executed
            sender,
        }
    }

    pub fn execute<F>(&self, f: F)
        where
            F: FnOnce() + Send + 'static
    {
        let job = Box::new(f);
        
        // Needs to send jobs wrapped in a Message::NewJob variant
        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");

        for _ in &mut self.workers {
            // Send one Terminate message for each worker
            self.sender.send(Message::Terminate).unwrap();
        }

        println!("Shutting down all workers.");

        // loop through the thread pool 
        for worker in &mut self.workers {  // Must use &mut because self is a mutable reference and we need to mutate `worker`
            println!("Shutting down worker {}", worker.id);
            
            // We're using `if let` to destructure the `Some` and get the thread
            // All workers currently have the Terminate message
            if let Some(thread) = worker.thread.take() {
                // Call join on the thread
                thread.join().unwrap();
            }
        }
    }
}

// Similar to Fn* traits except that it takes self: Box<Self> to take ownership of self
trait FnBox {
    fn call_box(self: Box<Self>);
}

// F implementst the FnOnce trait, which allows any FnOnce() closures to use our `call_box` method
impl<F: FnOnce()> FnBox for F {
    /// Take ownership of self and move the value out of Box<T>
    fn call_box(self: Box<F>) {
        // move the closure out of the Box<F> and call the closure
        (*self)()
    }
}

// A `Box` of anything that implements the `FnBox` trait
// This allows us to use `call_box` in `Worker` when we get the `Job` value instead of invoking the closure directly
type Job = Box<dyn FnBox + Send + 'static>;


struct Worker {
    id: usize,
    // By storing the thread inside an Option enum, we can take ownership of the thread (See impl Drop for ThreadPool)
    // JoinHandle<()> is an owned permission to join on a thread (block on its termination) 
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    // Contains a r
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        // Created by spawning a new thread using an empty closure
        let thread = thread::spawn(move || {
            loop {
                // Call lock on the message to acquire the mutex
                // Call unwrap to panic on any errors
                // Call recv to receive the Job from the channel
                // Call unwrap to panic on any errors
                let message = receiver.lock().unwrap().recv().unwrap();

                match message {
                    // Message is received by the channel
                    Message::NewJob(job) => {
                        println!("Worker {} got a job; executing.", id);
                        
                        // Dereference the value inside of Box
                        job.call_box();
                    },
                    // Break out of the loop
                    Message::Terminate => {
                        println!("Worker {} was told to terminate.", id);
                        break;
                    },
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

enum Message {
    NewJob(Job),
    Terminate,
}