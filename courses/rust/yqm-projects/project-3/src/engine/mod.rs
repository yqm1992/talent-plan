mod kvstore;
mod sled_engine;

pub use kvstore::KvStore;
pub use sled_engine::SledEngine;

use crate::*;

use std::fmt;

#[derive(Debug, Clone)]
pub enum EngineType {
    EngineKvStore,
    EngineSled,
}

const KVS_STR: &str = "kvs";
const SLED_STR: &str = "sled";

impl EngineType {
    pub fn from_str(s: &str) -> Result<EngineType> {
        match s {
            KVS_STR => Ok(EngineType::EngineKvStore),
            SLED_STR => Ok(EngineType::EngineSled),
            _ => Err(KvStoreError::OtherError(format!("invalid init engine type: {}", s).to_string())),
        }
    }
}

impl fmt::Display for EngineType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.clone() {
            EngineType::EngineKvStore => write!(f, "{}", KVS_STR),
            EngineType::EngineSled => write!(f, "{}",SLED_STR),
        }
    }
}

pub trait KvsEngine {

    /// Set value for the key in engine
    fn set(&mut self, key: String, value: String) -> Result<()>;

    /// Get value of the key from engine
    fn get(&mut self, key: String) -> Result<Option<String>>;

    /// Remove a key from the engine
    fn remove(&mut self, key: String) -> Result<()>;
}

