use std::{sync::{mpsc, Arc, Mutex}, thread};

pub struct ThreadPool {
    _workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
    receiver: mpsc::Receiver<()>,
    count: usize
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let mut workers = Vec::with_capacity(size);

        let receiver = Arc::new(Mutex::new(receiver));

        let (worker_sender, pool_receiver) = mpsc::channel();

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver), worker_sender.clone()));
        }

        ThreadPool { _workers: workers, sender, receiver: pool_receiver, count: 0 }
    }

    pub fn execute<F>(&mut self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.count += 1;
        self.sender.send(job).unwrap();
    }

    pub fn wait(&mut self) {
        while self.count > 0 {
            let _ = self.receiver.recv();
            self.count -= 1;
        }
    }
}

struct Worker {
    _id: usize,
    _thread: thread::JoinHandle<()>
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>, sender: mpsc::Sender<()>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv().unwrap();
            job();
            sender.send(()).unwrap()
        });

        Worker { _id: id, _thread: thread }
    }
}
