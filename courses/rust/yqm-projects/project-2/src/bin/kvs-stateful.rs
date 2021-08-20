use kvs::{KvStore};

fn main() {
    println!("Usage:");
    println!("\t1. set [key] [value]\n\t2. get [key]\n\t3. rm [key]\n\t3. exit\n");

    let usage = || {
        println!("\nInvalid input, usage:");
        println!("\t1. set [key] [value]\n\t2. get [key]\n\t3. rm [key]\n\t3. exit");
    };

    let mut kvstore = KvStore::open(".").unwrap();
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let vec: Vec<&str> = input.split("\n").next().unwrap().split(" ").filter(|x| { *x != "" }).collect();
        match vec.len() {
            1 => {
                match vec[0] {
                    "exit" => break,
                    _ => usage(),
                }
            },
            2 => {
                match vec[0] {
                    "get" => {
                        let key = vec[1].to_string();
                        match kvstore.get(key) {
                            Ok(Some(value)) => println!("{}", value),
                            Ok(None) => println!("Key not found"),
                            Err(e) => println!("{:?}", e)
                        }
                    },
                    "rm" => {
                        let key = vec[1].to_string();
                        match kvstore.remove(key) {
                            Ok(_) => {},
                            Err(_) => { println!("Key not found"); },
                        }
                    },
                    _ => usage(),
                }
            },
            3 => {
                match vec[0] {
                    "set" => {
                        let key = vec[1].to_string();
                        let value = vec[2].to_string();
                        kvstore.set(key, value).unwrap();
                    },
                    _ => usage(),
                }
            }
            _ => usage(),
        }
    }
}