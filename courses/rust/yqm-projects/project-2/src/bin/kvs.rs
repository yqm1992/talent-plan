extern crate clap;
use std::env;
use clap::{Arg, App, SubCommand};
use kvs::{KvStore};

fn main() {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Teaches argument parsing")
        .subcommand(SubCommand::with_name("get")
            .about("Get value of the key")
            .arg(Arg::with_name("key")
                .takes_value(true)
                .required(true)))
        .subcommand(SubCommand::with_name("set")
            .about("Set value for key")
            .arg(Arg::with_name("key")
                .takes_value(true)
                .required(true))
            .arg(Arg::with_name("value")
                .takes_value(true)
                .required(true)))
        .subcommand(SubCommand::with_name("rm")
            .about("Remove the key from database")
            .arg(Arg::with_name("key")
                .takes_value(true)
                .required(true)))
        .get_matches();

    // let mut kvstore = KvStore::open("data").unwrap();
    let mut kvstore = KvStore::open(".").unwrap();
    if let Some(subname) = matches.subcommand_name() {
        if let Some(submatches) = matches.subcommand_matches(subname) {
            match subname {
                "get" => {
                    let key = submatches.value_of("key").unwrap();
                    match kvstore.get(key.to_string()) {
                        Ok(Some(value)) => println!("{}", value),
                        Ok(None) => println!("Key not found"),
                        Err(e) => println!("{:?}", e)
                    }
                },
                "set" => {
                    let key = submatches.value_of("key").unwrap().to_owned();
                    let value = submatches.value_of("value").unwrap().to_owned();
                    kvstore.set(key, value).unwrap();
                },
                "rm"  => {
                    let key = submatches.value_of("key").unwrap();
                    match kvstore.remove(key.to_string()) {
                        Ok(_) => {},
                        Err(_) => { println!("Key not found"); std::process::exit(1); },
                    }
                },
                _     => unreachable!(),
            }
            // panic!("unimplemented");
            std::process::exit(0);
        } else {
            unreachable!()
        }
    } else {
        println!("{}", matches.usage());
        std::process::exit(1);
    }
}