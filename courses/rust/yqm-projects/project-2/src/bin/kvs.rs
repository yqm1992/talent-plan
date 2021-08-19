extern crate clap;
use std::env;
use clap::{Arg, App, SubCommand};

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

    if let Some(subname) = matches.subcommand_name() {
        if let Some(submatches) = matches.subcommand_matches(subname) {
            match subname {
                "get" => {
                    let _key = submatches.value_of("key").unwrap();
                },
                "set" => {
                    let _key = submatches.value_of("key").unwrap();
                    let _value = submatches.value_of("value").unwrap();
                },
                "rm"  => {
                    let _key = submatches.value_of("key").unwrap();
                },
                _     => unreachable!(),
            }
            // panic!("unimplemented");
            eprintln!("unimplemented");
            std::process::exit(1);
        } else {
            unreachable!()
        }
    } else {
        println!("{}", matches.usage());
        std::process::exit(1);
    }
}
