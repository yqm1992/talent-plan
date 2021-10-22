use sled;
use std::path::PathBuf;

use crate::*;
use bstr::ByteSlice;
use std::sync::{Arc, Mutex};

const SLED_ENGINE_NAME: &str = "sled";

#[derive(Clone)]
pub struct SledEngine {
    db: Arc<sled::Db>,
}

impl SledEngine {
    /// Open the sled engine
    pub fn open(path: impl Into<PathBuf>) -> Result<SledEngine> {
        let path_buf = path.into();
        let db = sled::open(path_buf).map_err(KvStoreError::SledError)?;
        Ok(SledEngine{db: Arc::new(db)})
    }
}

impl KvsEngine for SledEngine {
    fn set(&self, key: String, val: String) -> Result<()> {
        self.db.insert(key, val.as_bytes()).and_then(|_| Ok(())).map_err(KvStoreError::SledError)
    }

    fn get(&self, key: String) -> Result<Option<String>> {
        match self.db.get(key).map_err(KvStoreError::SledError)? {
            Some(val) => {
                match val.to_str() {
                    Ok(s) => Ok(Some(s.to_string())),
                    Err(e) => Err(KvStoreError::Utf8Error(e)),
                }
            },
            None => Ok(None),
        }
    }

    fn remove(&self, key: String) -> Result<()> {
        match self.db.remove(key).map_err(KvStoreError::SledError)? {
            Some(_) => {
                self.db.flush().and_then(|_| Ok(())).map_err(KvStoreError::SledError)?;
                Ok(())
            },
            None => Err(KvStoreError::KeyNotFound)
        }
    }

    fn get_name(&self) -> String {
        SLED_ENGINE_NAME.to_string()
    }
}