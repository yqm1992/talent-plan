#[derive(Debug)]
pub enum KvStoreError {
    IOError(std::io::Error),
    JSError(serde_json::Error),
    ParseError(core::num::ParseIntError),
    OtherError(String),
}

pub type Result<T> = std::result::Result<T, KvStoreError>;