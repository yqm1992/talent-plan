use std::env;
use clap::{Arg, App};
use slog::{Drain, Logger};
use kvs::*;
use std::path::PathBuf;
use std::str::FromStr;
use std::io::{BufReader, BufRead, Write};

const DEFAULT_ADDR: &str = "127.0.0.1:4000";
const INIT_ENGINE_FILE_NAME: &str = ".init_engine";
const TEMP_INIT_ENGINE_FILE_NAME: &str = ".temp_init_engine";
const KVSTORE_PATH: &str = "./kvs_data";
const SLED_PATH: &str = "./sled_data";

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

/// Get the init engine type of server
fn get_init_engine_type() -> Result<Option<EngineType>> {
    let path_buf = PathBuf::from_str(INIT_ENGINE_FILE_NAME).unwrap();
    if ! path_buf.exists() {
        return Ok(None);
    }
    let file = std::fs::File::open(path_buf).unwrap();
    let mut reader = BufReader::new(file);
    let mut buf = String::new();
    reader.read_line(&mut buf).map_err(KvStoreError::IOError)?;
    EngineType::from_str(buf.as_str()).and_then(|t| Ok(Some(t)))
}

/// Write the init engine type to the storage
fn write_init_engine_type(engine_type :EngineType) -> Result<()> {
    let temp_init_file = TEMP_INIT_ENGINE_FILE_NAME;
    let init_file = INIT_ENGINE_FILE_NAME;
    let path_buf = PathBuf::from_str(temp_init_file).unwrap();
    let mut file = std::fs::File::create(path_buf).map_err(KvStoreError::IOError)?;

    match engine_type {
        EngineType::EngineKvStore => file.write_all("kvs".as_bytes()).map_err(KvStoreError::IOError)?,
        EngineType::EngineSled => file.write_all("sled".as_bytes()).map_err(KvStoreError::IOError)?,
    };
    std::fs::rename(temp_init_file, init_file).map_err(KvStoreError::IOError)
}

/// Check if selected engine matches with init engine, and return the init engine type
fn check_engine_type(engine_type: EngineType) -> Result<Option<EngineType>> {
    let opt_init_engine_type = get_init_engine_type()?;
    if let Some(t) = opt_init_engine_type.clone() {
        match (t.clone(), engine_type.clone()) {
            (EngineType::EngineKvStore, EngineType::EngineKvStore) => Ok(Some(EngineType::EngineKvStore)),
            (EngineType::EngineSled, EngineType::EngineSled) => Ok(Some(EngineType::EngineSled)),
            _ => return Err(KvStoreError::OtherError(format!("current selected engine [{}] mismatches with init engine [{}]", engine_type, t).to_string())),
        }
    } else {
        Ok(None)
    }
}

fn main() -> Result<()> {
    let log = setup_logger();
    let (addr, engine_type) = get_match();
    let init_engine_type = check_engine_type(engine_type.clone())?;
    match init_engine_type {
        Some(_) => {},
        None => write_init_engine_type(engine_type.clone())?,
    }
    let thread_pool = SharedQueueThreadPool::new(4).unwrap();

    match engine_type {
        EngineType::EngineKvStore => KvsServer::run(log, KvStore::open(KVSTORE_PATH)?, thread_pool, addr),
        EngineType::EngineSled => KvsServer::run(log, SledEngine::open(SLED_PATH)?, thread_pool, addr),
    }
}