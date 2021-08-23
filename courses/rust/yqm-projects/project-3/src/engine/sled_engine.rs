use sled;
use std::path::PathBuf;

use crate::*;
use bstr::ByteSlice;

pub struct SledEngine {
    db: sled::Db,
}

impl SledEngine {
    /// Open the sled engine
    pub fn open(path: impl Into<PathBuf>) -> Result<SledEngine> {
        let path_buf = path.into();
        let db = sled::open(path_buf).map_err(KvStoreError::SledError)?;
        Ok(SledEngine{db})
    }
}

impl KvsEngine for SledEngine {
    fn set(&mut self, key: String, val: String) -> Result<()> {
        self.db.insert(key, val.as_bytes()).and_then(|_| Ok(())).map_err(KvStoreError::SledError)
        // self.db.flush().and_then(|_| Ok(())).map_err(KvStoreError::SledError)
    }

    fn get(&mut self, key: String) -> Result<Option<String>> {
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

    fn remove(&mut self, key: String) -> Result<()> {
        match self.db.remove(key).map_err(KvStoreError::SledError)? {
            Some(_) => {
                self.db.flush().and_then(|_| Ok(())).map_err(KvStoreError::SledError)?;
                Ok(())
            },
            None => Err(KvStoreError::KeyNotFound)
        }
    }
}