use std::thread;
use std::sync::mpsc;
use std::sync::Mutex;
use std::sync::Arc;

/// A thread pool is a group of spawned threads that are waiting and ready to handle a task. When the program receives a new task, it assigns one of the threads in the pool to the task, and that thread will process the task. The remaining threads in the pool are available to handle any other tasks that come in while the first thread is processing.
pub struct ThreadPool {
    // Each worker holds on to the receiving side of the channel
    workers: Vec<Worker>,
    // holds on to the sending side of the channel
    sender: mpsc::Sender<Message>,
}

impl ThreadPool {
    /// Create a new ThreadPool with a configurable number of threads
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender,
        }
    }
    /// `execute` takes a closure it’s given and gives it to an idle thread in the pool to run
    /// 
    /// execute method will send the job it wants to execute down the sending side of the channel.
    pub fn execute<F>(&self, f: F)
        where
            F: FnOnce() + Send + 'static
    {
        let job = Box::new(f);
        
        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

/// Drop trait to call join on each of the threads in the pool so they can finish the requests they’re working on before closing.\
/// 
/// /// Joining each thread when the thread pool goes out of scope
impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");

        for _ in &mut self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        println!("Shutting down all workers.");

        for worker in &mut self.workers { 
            println!("Shutting down worker {}", worker.id);
            
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

/// Adds the ability to call the closure inside Box
trait FnBox {
    fn call_box(self: Box<Self>);
}

/// This means that any FnOnce() closures can use our call_box method
impl<F: FnOnce()> FnBox for F {
    /// call_box uses (*self)() to move the closure out of the Box<T> and call the closure
    fn call_box(self: Box<F>) {
        (*self)()
    }
}

/// Type alias for a trait object that holds the type of closure that execute receives
type Job = Box<dyn FnBox + Send + 'static>;

/// A Worker Struct is responsible for sending code from the ThreadPool to a Thread
/// 
/// The id is a unique number to distinguish between threads
/// 
/// The thread stores a single JoinHandle() instance wrapped in an Option enum 
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// Worker will loop over its receiving side of the channel and execute the closures of any jobs it receives
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                let message = receiver.lock().unwrap().recv().unwrap();

                match message {
                    Message::NewJob(job) => {
                        println!("Worker {} got a job; executing.", id);
                        job.call_box();
                    },
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

/// This Message enum will either be a NewJob variant that holds the Job the thread should run, or it will be a Terminate
/// variant that will cause the thread to exit its loop and stop.
enum Message {
    NewJob(Job),
    Terminate,
}