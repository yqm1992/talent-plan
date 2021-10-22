use std::io::{Read, Write, BufReader, BufWriter};
use std::net::{TcpListener, TcpStream, SocketAddr};
use slog::{Logger, info, error};

use crate::*;

pub struct KvsServer<E: KvsEngine, P: ThreadPool> {
    log: Logger,
    engine: E,
    thread_pool: P,
    listener: TcpListener,
    info: ServerInfo,
}

#[derive(Debug)]
struct ServerInfo {
    addr: String,
    engine_name: String,
    version: String,
}

impl<E: KvsEngine, P: ThreadPool> KvsServer<E, P> {

    pub fn run(log: Logger, engine: E, thread_pool: P, addr: String) -> Result<()> {
        let info = ServerInfo{addr: addr.clone(), engine_name: engine.get_name(), version: env!("CARGO_PKG_VERSION").to_string()};
        let listener = TcpListener::bind(addr.clone()).map_err(KvStoreError::IOError)?;
        let mut server = KvsServer{log, engine, thread_pool, listener, info};
        server.start()
    }

    /// Server start to service
    pub fn start(&mut self) -> Result<()> {
        info!(self.log, "Server start, {:?}", self.info);
        loop {
            let new_client =  self.listener.accept().map_err(KvStoreError::IOError)?;
            let log = self.log.clone();
            let engine = self.engine.clone();
            self.thread_pool.spawn(move || {
                if let Err(e) = handle_connection(engine, &log, new_client) {
                    error!(log, "Failed to handle connection, {:?}", e);
                }
            });
        }
        Ok(())
    }
}

/// Handle connection from the client
fn handle_connection<E: KvsEngine>(engine: E, log: &Logger, new_client: (TcpStream, SocketAddr)) -> Result<()> {
    let stream = new_client.0;
    let cli_addr = new_client.1;
    info!(log, "New connection from {}", cli_addr);

    let mut reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);

    let command_iter = serde_json::Deserializer::from_reader(reader).into_iter::<Command>();
    for item in command_iter {
        let cmd = item.map_err(KvStoreError::JSError)?;
        info!(log, "Receive command [{:?}] from [{:?}]", cmd, cli_addr);

        let resp = exec_command(&engine, log, cmd);

        respond_to_client(&mut writer, &resp)?;
        info!(log, "Respond [{:?}] to [{:?}]", resp, cli_addr);
    }
    Ok(())
}

/// Execute the command received from client
fn exec_command<E: KvsEngine>(engine: &E, log: &Logger, cmd: Command) -> Response {
    let server_error_str = "Server internal error".to_string();
    let resp = match cmd {
        Command::Get(key) => {
            match engine.get(key) {
                Ok(val) => Response::Ok(CommandResponse::GetResponse(val)),
                Err(e) => {
                    error!(log, "Failed to get the value of key, {:?}", e);
                    Response::Err(server_error_str)
                },
            }
        },
        Command::Set(key, val) => {
            match engine.set(key, val) {
                Ok(_) => Response::Ok(CommandResponse::SetResponse),
                Err(e) => {
                    error!(log, "Failed to set value for key, {:?}", e);
                    Response::Err(server_error_str)
                },
            }
        },
        Command::Remove(key) => {
            match engine.remove(key) {
                Ok(_) => Response::Ok(CommandResponse::RemoveResponse),
                Err(e) => {
                    match e {
                        KvStoreError::KeyNotFound => Response::Err("Key not found".to_string()),
                        _ => {
                            error!(log, "Failed to remove key, {:?}", e);
                            Response::Err(server_error_str)
                        },
                    }
                },
            }
        }
    };
    resp
}

fn respond_to_client(writer: &mut BufWriter<&TcpStream>, resp: &Response) -> Result<()> {
    let vec = serde_json::to_vec(resp).map_err(KvStoreError::JSError)?;
    writer.write_all(&vec).map_err(KvStoreError::IOError)?;
    writer.flush().map_err(KvStoreError::IOError)
}