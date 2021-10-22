#[derive(Debug)]
pub enum KvStoreError {
    IOError(std::io::Error),
    JSError(serde_json::Error),
    ParseError(core::num::ParseIntError),
    SledError(sled::Error),
    Utf8Error(bstr::Utf8Error),
    KeyNotFound,
    OtherError(String),
}

pub type Result<T> = std::result::Result<T, KvStoreError>;