use thread_pool::{ThreadPool, Job};
use std::sync::{Condvar, Arc, RwLock, Barrier};
use std::cell::Cell;

fn test_panic() {
    panic!("panic this thread!");
}

fn test_thread_pool() {
    let thread_pool = ThreadPool::new(4).unwrap();
    thread_pool.execute(Box::new(move || {
        println!("Hello World!")
    }));
    let mut buf = String::new();
    while std::io::stdin().read_line(&mut buf).unwrap() != 0 {
        match buf.as_str() {
            "exit\n" => break,
            "panic\n" => thread_pool.execute(Box::new(test_panic)),
            _ => {
                let s = buf.clone();
                let job = Box::new(move || {
                    println!("\tinput: {}", s);
                });
                thread_pool.execute(job);
            },
        }
        buf.clear();
    }
    // thread_pool.execute(Box::new(test_panic));
}

fn main() {
    test_thread_pool();
    // ttt();
}


fn ttt() {
    use std::panic;

    let result = panic::catch_unwind(|| {
        println!("hello!");
    });
    assert!(result.is_ok());

    let result = panic::catch_unwind(|| {
        panic!("oh no!");
    });
    assert!(result.is_err());

    println!("world!");

}
