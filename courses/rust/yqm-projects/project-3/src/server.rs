use std::io::{Read, Write, BufReader, BufRead};
use std::net::{TcpListener, TcpStream, SocketAddr};
use slog::{Logger, info, error};

use crate::*;
use std::path::PathBuf;
use std::str::FromStr;

pub struct KvsServer {
    log: Logger,
    engine: Box<dyn KvsEngine>,
    listener: TcpListener,
    info: ServerInfo,
}

#[derive(Debug)]
struct ServerInfo {
    addr: String,
    engine_name: String,
    version: String,
}

const INIT_ENGINE_FILE_NAME: &str = ".init_engine";
const TEMP_INIT_ENGINE_FILE_NAME: &str = ".temp_init_engine";

impl KvsServer {

    /// Create a kv server
    pub fn new(log: Logger, engine_type: EngineType, addr: String) -> Result<KvsServer> {
        let info = ServerInfo{addr: addr.clone(), engine_name: engine_type.to_string(), version: env!("CARGO_PKG_VERSION").to_string()};
        let listener = TcpListener::bind(addr.clone()).map_err(|e| {
            error!(log, "Failed to create listener at addr {:?}, {:?}", addr, e);
            KvStoreError::IOError(e)
        })?;
        let engine = Self::new_engine(engine_type.clone())?;
        let server_result = Ok(KvsServer{log, engine, listener, info});
        server_result
    }

    /// Server start to service
    pub fn start(&mut self) {
        info!(self.log, "Server start, {:?}", self.info);
        loop {
            match self.listener.accept() {
                Ok(new_client) => {
                    self.handle_connection(new_client);
                },
                Err(e) => {
                    error!(self.log, "Failed to get connection, {}", e.to_string());
                    break;
                },
            }
        }
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

    /// Create engine by the given engine type
    fn new_engine(engine_type: EngineType) -> Result<Box<dyn KvsEngine>> {
        let opt_init_engine_type = KvsServer::get_init_engine_type()?;
        if let Some(t) = opt_init_engine_type.clone() {
            match (t.clone(), engine_type.clone()) {
                (EngineType::EngineKvStore, EngineType::EngineKvStore) => {},
                (EngineType::EngineSled, EngineType::EngineSled) => {},
                _ => return Err(KvStoreError::OtherError(format!("current selected engine [{}] mismatches with init engine [{}]", engine_type, t).to_string())),
            }
        }

        let result_engine: Result<Box<dyn KvsEngine>> = match engine_type {
            EngineType::EngineKvStore => {
                let kvstore = KvStore::open("./kvs_data")?;
                Ok(Box::new(kvstore))
            },
            EngineType::EngineSled => {
                let sled_engine = SledEngine::open("./sled_data")?;
                Ok(Box::new(sled_engine))
            }
        };

        if let None = opt_init_engine_type {
            Self::write_init_engine_type(engine_type)?;
        }
        result_engine
    }

    /// Handle connection from the client
    fn handle_connection(&mut self, new_client: (TcpStream, SocketAddr)) {
        let mut conn = new_client.0;
        let cli_addr = new_client.1;
        info!(self.log, "New connection from {}", cli_addr);

        let mut buf = [0;4096];
        let len = match conn.read(&mut buf) {
            Ok(n) => n,
            Err(e) => {
                error!(self.log, "Failed to read connection {}, {:?}", cli_addr, e);
                return;
            },
        };

        let cmd = match serde_json::from_slice::<Command>(&buf[..len]) {
            Ok(cmd) => cmd,
            Err(e) => {
                error!(self.log, "Failed to decode command from stream, {:?}", e);
                return;
            },
        };
        info!(self.log, "Receive command [{:?}] from [{:?}]", cmd, cli_addr);

        let resp = self.exec_remote_command(cmd);

        let vec = match serde_json::to_vec(&resp) {
            Ok(vec) => vec,
            Err(e) => {
                error!(self.log, "Failed to serialize command, {:?}", e);
                return;
            },
        };

        if let Err(e) = conn.write_all(&vec) {
            error!(self.log, "Failed to write to connection {}, {:?}", cli_addr, e);
        } else {
            info!(self.log, "Respond [{:?}] to [{:?}]", resp, cli_addr);
            // info!(self.log, "\t[{:?}] to [{:?}]", vec.to_str().unwrap(), cli_addr);
        }
    }

    /// Execute the command received from client
    fn exec_remote_command(&mut self, cmd: Command) -> Response {
        let server_error_str = "Server internal error".to_string();
        let resp = match cmd {
            Command::Get(key) => {
                match self.engine.get(key) {
                    Ok(val) => Response::Ok(CommandResponse::GetResponse(val)),
                    Err(e) => {
                        error!(self.log, "Failed to get the value of key, {:?}", e);
                        Response::Err(server_error_str)
                    },
                }
            },
            Command::Set(key, val) => {
                match self.engine.set(key, val) {
                    Ok(_) => Response::Ok(CommandResponse::SetResponse),
                    Err(e) => {
                        error!(self.log, "Failed to set value for key, {:?}", e);
                        Response::Err(server_error_str)
                    },
                }
            },
            Command::Remove(key) => {
                match self.engine.remove(key) {
                    Ok(_) => Response::Ok(CommandResponse::RemoveResponse),
                    Err(e) => {
                        match e {
                            KvStoreError::KeyNotFound => Response::Err("Key not found".to_string()),
                            _ => {
                                error!(self.log, "Failed to remove key, {:?}", e);
                                Response::Err(server_error_str)
                            },
                        }
                    },
                }
            }
        };
        resp
    }
}