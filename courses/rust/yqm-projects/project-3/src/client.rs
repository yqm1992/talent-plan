use std::io::{Write, Read};
use std::net::TcpStream;

use crate::*;

pub struct KvsClient {
    stream: TcpStream,
}

impl KvsClient {
    /// Create a client to communicate with the server
    pub fn new(addr: String) -> Result<KvsClient> {
        let stream = match TcpStream::connect(addr.clone()) {
            Ok(stream) => stream,
            Err(e) => {
                eprintln!("Failed to connect the server {:?}, {:?}", addr, e);
                return  Err(KvStoreError::IOError(e))
            },
        };
        Ok(KvsClient{stream})
    }

    /// Ask the server to execute the given command
    pub fn exec_remote_cmd(&mut self, cmd: Command) {
        // let mut buf = String::new();
        self.stream.write_all(&serde_json::to_vec(&cmd).unwrap()).unwrap();

        let mut buf = [0;4096];

        let len = self.stream.read(&mut buf).unwrap();
        let resp: Response = serde_json::from_slice(&buf[..len]).unwrap();

        if ! Self::check_response_valid(cmd.clone(), resp.clone()) {
            eprintln!("Response {:?} mismatches the command {:?}", resp, cmd);
            return;
        }

        match resp {
            Ok(cmd_resp) => {
                match cmd_resp {
                    CommandResponse::GetResponse(opt) => {
                        if let Some(val) = opt {
                            println!("{}", val);
                        } else {
                            println!("Key not found");
                        }
                    },
                    _ => {},
                }
            },
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
    }

    /// Check if the response of server is consistent with the command
    fn check_response_valid(cmd: Command, resp: Response) -> bool {
        if let Err(_) = resp {
            return true;
        }
        let cmd_resp = resp.unwrap();
        match (cmd, cmd_resp) {
            (Command::Get(_), CommandResponse::GetResponse(_)) => true,
            (Command::Set(_, _), CommandResponse::SetResponse) => true,
            (Command::Remove(_), CommandResponse::RemoveResponse) => true,
            _ => false,
        }
    }
}