use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    Get(String),
    Set(String, String),
    Remove(String),
}