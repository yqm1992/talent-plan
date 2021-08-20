use std::collections::HashMap;
use std::path::PathBuf;
use std::fs::File;
use std::io::Write;


#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    Get(String),
    Set(String, String),
    Remove(String),
}

#[derive(Debug)]
pub enum KvStoreError {
    IOError(std::io::Error),
    JSError(serde_json::Error),
    ParseError(core::num::ParseIntError),
    OtherError(String),
}

pub type Result<T> = std::result::Result<T, KvStoreError>;

/// CommandPos is used to record the file position of a command
#[derive(Debug)]
struct CommandPos {
    /// The log file's index
    index: u64,
    /// The offset of value
    start_pos: u64,
    /// The length of value
    len: u64,
}

/// This represents a log file for KvStore current writing
struct LogWriteHandle {
    /// The log file's index
    index: u64,
    /// The log file's write handle
    file: File,
    /// The position to write next time
    next_pos: u64,
}

pub struct KvStore {
    /// The root directory
    root_dir_path_buf: PathBuf,
    /// The in-memory index from key to log pointer
    kv_map: HashMap<String, CommandPos>,
    /// The handle of current writing log file
    log_write_handle: LogWriteHandle,
}

impl KvStore {
    /// Open the database
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        Err(KvStoreError::OtherError("no implementation".to_string()))
    }
    /// Get the value of key from KvStore
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        Err(KvStoreError::OtherError("no implementation".to_string()))
    }
    /// Set value for key in KvStore
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        Err(KvStoreError::OtherError("no implementation".to_string()))
    }
    /// Remove a key from KvStore
    pub fn remove(&mut self, key: String) -> Result<()> {
        Err(KvStoreError::OtherError("no implementation".to_string()))
    }
}
