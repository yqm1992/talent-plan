use std::env;
use clap::{Arg, App, SubCommand};
use kvs::*;

const DEFAULT_ADDR: &str = "127.0.0.1:4000";

fn get_match() -> (Command, String) {
    let addr_arg = Arg::with_name("addr")
        .long("addr")
        .value_name("IP-PORT")
        .takes_value(true)
        .required(true)
        .default_value(DEFAULT_ADDR);

    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Teaches argument parsing")
        .subcommand(SubCommand::with_name("get")
            .about("Get value of the key")
            .arg(Arg::with_name("key")
                .takes_value(true)
                .required(true))
            .arg(addr_arg.clone()))
        .subcommand(SubCommand::with_name("set")
            .about("Set value for key")
            .arg(Arg::with_name("key")
                .takes_value(true)
                .required(true))
            .arg(Arg::with_name("value")
                .takes_value(true)
                .required(true))
            .arg(addr_arg.clone()))
        .subcommand(SubCommand::with_name("rm")
            .about("Remove the key from database")
            .arg(Arg::with_name("key")
                .takes_value(true)
                .required(true))
            .arg(addr_arg.clone()))
        .get_matches();

    let opt_cmd: Option<Command>;
    let opt_addr: Option<String>;
    if let Some(subname) = matches.subcommand_name() {
        if let Some(submatches) = matches.subcommand_matches(subname) {
            match subname {
                "get" => {
                    let key = submatches.value_of("key").unwrap().to_owned();
                    let addr = submatches.value_of("addr").unwrap().to_owned();
                    opt_cmd = Some(Command::Get(key));
                    opt_addr = Some(addr);
                },
                "set" => {
                    let key = submatches.value_of("key").unwrap().to_owned();
                    let value = submatches.value_of("value").unwrap().to_owned();
                    let addr = submatches.value_of("addr").unwrap().to_owned();
                    opt_cmd = Some(Command::Set(key, value));
                    opt_addr = Some(addr);
                },
                "rm"  => {
                    let key = submatches.value_of("key").unwrap().to_owned();
                    let addr = submatches.value_of("addr").unwrap().to_owned();
                    opt_cmd = Some(Command::Remove(key));
                    opt_addr = Some(addr);
                },
                _     => unreachable!(),
            }
        } else {
            unreachable!()
        }
    } else {
        println!("{}", matches.usage());
        std::process::exit(1);
    }
    let cmd = opt_cmd.unwrap();
    let addr = opt_addr.unwrap();
    (cmd, addr)
}

fn main() -> Result<()> {
    let (cmd, addr) = get_match();
    match KvsClient::new(addr) {
        Ok(client) => client.exec_remote_cmd(cmd),
        Err(e) => {
            eprintln!("{:?}", e);
            std::process::exit(1);
        }
    }
}