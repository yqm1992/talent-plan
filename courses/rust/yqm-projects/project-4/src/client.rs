use std::io::{Write, BufWriter, BufReader};
use std::net::{TcpStream};

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
    pub fn exec_remote_cmd(&self, cmd: Command) -> Result<()> {
        let mut reader = BufReader::new(&self.stream);
        let mut writer = BufWriter::new(&self.stream);

        Self::send_to_server(&mut writer, &cmd)?;
        let resp = Self::get_response(&mut reader)?;
        Self::check_response(cmd.clone(), resp.clone()).unwrap();
        Self::display_response(resp);
        Ok(())
    }

    /// Send command to server
    fn send_to_server(writer: &mut BufWriter<&TcpStream>, cmd: &Command) -> Result<()> {
        let vec = serde_json::to_vec(cmd).map_err(KvStoreError::JSError)?;
        writer.write_all(&vec).map_err(KvStoreError::IOError)?;
        writer.flush().map_err(KvStoreError::IOError)
    }

    /// Get response from server
    fn get_response(reader: &mut BufReader<&TcpStream>) -> Result<Response> {
        let mut command_iter = serde_json::Deserializer::from_reader(reader).into_iter::<Response>();
        let resp = command_iter.next().unwrap().map_err(KvStoreError::JSError)?;
        Ok(resp)
    }

    /// Check if the response of server is consistent with the command
    fn check_response(cmd: Command, resp: Response) -> Result<()> {
        if let Err(_) = resp {
            return Ok(());
        }
        let cmd_resp = resp.map_err(KvStoreError::OtherError)?;
        match (cmd.clone(), cmd_resp.clone()) {
            (Command::Get(_), CommandResponse::GetResponse(_)) => Ok(()),
            (Command::Set(_, _), CommandResponse::SetResponse) => Ok(()),
            (Command::Remove(_), CommandResponse::RemoveResponse) => Ok(()),
            _ => {
                let s = format!("Response {:?} mismatches the command {:?}", cmd_resp, cmd);
                return Err(KvStoreError::OtherError(s));
            },
        }
    }

    /// Display the response from server
    fn display_response(resp: Response) {
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
            },
        }
    }
}
