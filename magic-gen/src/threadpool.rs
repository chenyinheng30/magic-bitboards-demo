use std::{
    sync::{
        mpsc,
        Arc, Mutex,
    },
    thread,
};

pub struct ThreadPool {
    _workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let mut workers = Vec::with_capacity(size);

        let receiver = Arc::new(Mutex::new(receiver));

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            _workers: workers,
            sender,
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}

struct Worker {
    _id: usize,
    _thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv();
            match job {
                Ok(job) => job(),
                Err(_) => break,
            }
        });

        Worker {
            _id: id,
            _thread: thread,
        }
    }
}

pub mod future {
    use std::sync::mpsc::{self, Receiver, Sender};

    pub struct Binder<T> {
        sender: Sender<T>,
    }

    pub fn tunnel<T>() -> (Binder<T>, Receiver<T>) {
        let (sender, receiver) = mpsc::channel::<T>();
        return (Binder { sender }, receiver);
    }

    impl<T> Binder<T> {
        pub fn bind<F>(&self, function: F)
        where
            F: FnOnce() -> T,
        {
            let effect = function();
            self.sender.send(effect).unwrap();
        }
    }
}
