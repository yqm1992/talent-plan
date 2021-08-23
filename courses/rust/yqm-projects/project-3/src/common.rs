use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Command {
    Get(String),
    Set(String, String),
    Remove(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CommandResponse {
    GetResponse(Option<String>),
    SetResponse,
    RemoveResponse,
}

/// The result of server's response to the client
pub type Response = std::result::Result<CommandResponse, String>;