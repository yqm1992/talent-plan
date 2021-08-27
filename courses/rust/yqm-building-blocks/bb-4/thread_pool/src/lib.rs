use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc;
use std::thread;
use std::sync::mpsc::{Receiver, Sender};


pub type Job = Box<dyn FnOnce() + Send + 'static>;

struct Defender {
    id: u8,
    rx: Arc<Mutex<Receiver<ThreadPoolMessage>>>,
}

enum ThreadPoolMessage {
    RunJob(Job),
    Shutdown,
}

struct Worker {
    id: u8,
    handle_opt: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(id: u8, rx: Arc<Mutex<Receiver<ThreadPoolMessage>>>) -> Self {
        let work_func = move || {
            loop {
                let core =  || -> bool {
                    let mut stopped = false;
                    let msg = rx.lock().unwrap().recv().unwrap();
                    match msg {
                        ThreadPoolMessage::RunJob(job) => {
                            println!("thread({}) receives a new job", id);
                            job();
                            stopped = false
                        },
                        ThreadPoolMessage::Shutdown => stopped = true,
                    }
                    stopped
                };
                match std::panic::catch_unwind(core) {
                    Ok(stopped) => {
                        if stopped {
                            break;
                        }
                    },
                    Err(e) => println!("thread({}) caught a panic, continue do the job", id),
                }
            }
            println!("thread({}) receives stop message", id);
        };
        let handle = thread::spawn(work_func);
        println!("create a new thread: {}", id);
        Worker{id, handle_opt: Some(handle)}
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        let handle = self.handle_opt.take().unwrap();
        let ret_val = handle.join();
        println!("thread({}) exit, status: {:?}", self.id, ret_val);
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Sender<ThreadPoolMessage>,
}

impl ThreadPool {
    pub fn new(num: u8) -> Result<Self, String> {
        if num == 0 {
            return Err("thread num can not be 0".to_string());
        }
        let mut workers = Vec::new();
        let (sender, receiver) = mpsc::channel::<ThreadPoolMessage>();
        let rx = Arc::new(Mutex::new(receiver));
        for i in 0..num {
            workers.push(Worker::new(i, rx.clone()));
        }
        Ok(ThreadPool{workers, sender})
    }
    pub fn execute(&self, job: Job) {
        let msg = ThreadPoolMessage::RunJob(job);
        self.sender.send(msg);
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for worker in self.workers.iter_mut() {
            self.sender.send(ThreadPoolMessage::Shutdown);
        }
    }
}

fn spawn_in_pool(workers: Arc<Mutex<Vec<Worker>>>, receiver: Arc<Mutex<Receiver<ThreadPoolMessage>>>) {
    let mut vec = workers.lock().unwrap();
    let worker = Worker::new(0, receiver);
    vec.push(worker);
}
