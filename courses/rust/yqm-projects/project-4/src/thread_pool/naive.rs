use crate::*;

pub struct NaiveThreadPool {
    num: u32,
}

impl ThreadPool for NaiveThreadPool {
    fn new(threads: u32) -> Result<NaiveThreadPool> {
        if threads == 0 {
            return Err(KvStoreError::OtherError("thread num can not be 0".to_string()));
        }
        Ok(NaiveThreadPool{num: threads})
    }
    fn spawn<F>(&self, job: F) where F: FnOnce() + Send + 'static {
        std::thread::spawn(job);
    }
}