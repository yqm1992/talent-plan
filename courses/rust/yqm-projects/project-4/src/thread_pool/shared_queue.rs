use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc;
use std::thread;
use std::sync::mpsc::{Receiver, Sender};
use crate::*;


pub type Job = Box<dyn FnOnce() + Send + 'static>;

enum ThreadPoolMessage {
    RunJob(Job),
    Shutdown,
}

struct Worker {
    id: u32,
    handle_opt: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(id: u32, rx: Arc<Mutex<Receiver<ThreadPoolMessage>>>) -> Self {
        let work_func = move || {
            loop {
                let core =  || -> bool {
                    let msg = rx.lock().unwrap().recv().unwrap();
                    let stopped= match msg {
                        ThreadPoolMessage::RunJob(job) => {
                            println!("new job");
                            job();
                            false
                        },
                        ThreadPoolMessage::Shutdown => true,
                    };
                    stopped
                };
                match std::panic::catch_unwind(core) {
                    Ok(stopped) => {
                        if stopped {
                            break;
                        }
                    },
                    Err(_) => println!("thread({}) caught a panic, continue do the job", id),
                }
            }
        };
        let handle = thread::spawn(work_func);
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

pub struct SharedQueueThreadPool {
    workers: Vec<Worker>,
    sender: Sender<ThreadPoolMessage>,
}

impl ThreadPool for SharedQueueThreadPool {
    fn new(threads: u32) -> Result<Self>
        where Self: Sized
    {
        if threads == 0 {
            return Err(KvStoreError::OtherError("thread num can not be 0".to_string()));
        }
        let mut workers = Vec::new();
        let (sender, receiver) = mpsc::channel::<ThreadPoolMessage>();
        let rx = Arc::new(Mutex::new(receiver));
        for i in 0..threads {
            workers.push(Worker::new(i, rx.clone()));
        }
        Ok(SharedQueueThreadPool{workers, sender})
    }
    fn spawn<F>(&self, job: F)
        where F: FnOnce() + Send + 'static
    {
        self.sender.send(ThreadPoolMessage::RunJob(Box::new(job)));
    }
}

impl Drop for SharedQueueThreadPool {
    fn drop(&mut self) {
        for _ in self.workers.iter_mut() {
            self.sender.send(ThreadPoolMessage::Shutdown);
        }
    }
}