use std::env;
use clap::{Arg, App};
use slog::{Drain, Logger};
use kvs::*;

const DEFAULT_ADDR: &str = "127.0.0.1:4000";

/// Set up logger, and attach it to stderr
fn setup_logger() -> Logger {
    let decorator = slog_term::TermDecorator::new().stderr().force_color().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    slog::Logger::root(drain, slog::o!())
}

/// Parse command line, return addr and engine_name
fn get_match() -> (String, EngineType) {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("KvServer")
        .arg(
            Arg::with_name("addr")
                .long("addr")
                .value_name("IP-PORT")
                .takes_value(true)
                .required(true)
                .default_value(DEFAULT_ADDR)
        )
        .arg(
            Arg::with_name("engine")
                .long("engine")
                .value_name("ENGINE-NAME")
                .takes_value(true)
                .required(true)
                .default_value("kvs")
        )
        .get_matches();

    let addr = matches.value_of("addr").unwrap().to_string();
    let engine_name = matches.value_of("engine").unwrap().to_string();

    let engine_type = match engine_name.as_str() {
        "kvs" => EngineType::EngineKvStore,
        "sled" => EngineType::EngineSled,
        _ => {
            eprintln!("unsupported engine: {}, only support two engines: kvs, sled", engine_name);
            std::process::exit(1);
        },
    };

    (addr, engine_type)
}

fn main() -> Result<()> {
    let log = setup_logger();
    let (addr, engine_type) = get_match();
    let mut server = KvsServer::new(log, engine_type, addr)?;
    server.start();
    Ok(())
}