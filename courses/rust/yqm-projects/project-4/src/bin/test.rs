use std::thread;
use std::env::current_dir;
use kvs::{KvsEngine, KvStore, Result};
use std::cell::RefCell;
use std::sync::{Mutex, Arc};

fn main() -> Result<()> {
    let store_for_write = KvStore::open(".")?;
    let store_for_read = store_for_write.clone();
    let iter_num = 100000;

    let write_handle = thread::spawn( move || {
        for _ in 0..iter_num {
            if let Err(e) = store_for_write.set("key".to_owned(), "value".to_owned()) {
                println!("SetError: {:?}", e);
            }
        }
        println!("write finished!");
    });

    let read_handle = thread::spawn( move || {
        for _ in 0..iter_num {
            if let Err(e) = store_for_read.get("key".to_owned()) {
                println!("Get_Error: {:?}", e);
            }
        }
        println!("read finished!");
    });
    write_handle.join().unwrap();
    read_handle.join().unwrap();
    Ok(())
}

fn test() {
    let a = Some(32);
    let b = a.map(|x| {x*x});
    let c = a.and_then(|x| {Some(0_u32)});
    let d = RefCell::new(32);
    let x = d.borrow_mut();
    *x = 64;
    let t = Box::new(32);
    let xxx = Arc::new(32);
}

