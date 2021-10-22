use std::collections::{HashMap, BTreeMap, VecDeque};
use std::path::PathBuf;
use serde_json::Deserializer;
use std::fs;
use std::fs::{File, DirEntry};
use std::io::{Write, Seek, SeekFrom};

use crate::*;
use std::sync::{Arc, Mutex};
use crate::engine::KVS_STR;

const KVSTORE_ENGINE_NAME: &str = "kvs";

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

impl CommandPos {
    /// Read command from the log file
    ///
    /// return command
    pub fn get_command_with_pool(&self, file_pool: &mut FilePool) -> Result<Command> {
        let file = file_pool.open(self.index)?;
        let mut ref_file = file.as_ref();
        ref_file.seek(SeekFrom::Start(self.start_pos)).map_err(KvStoreError::IOError)?;
        let mut stream = Deserializer::from_reader(ref_file).into_iter::<Command>();
        stream.next().ok_or(KvStoreError::OtherError("Got None from storage by cmd_pos".to_string()))?.map_err(KvStoreError::JSError)
    }
}

/// FilePool is used to cache the open files
struct FilePool {
    /// The root directory
    root_dir_path_buf: PathBuf,
    /// The max number of cached open files
    max_open_files: u64,
    /// The map from file index to File
    files_map: BTreeMap<u64, Arc<File>>,
    /// The queue of opened files
    index_queue: VecDeque<u64>,
}

impl FilePool {
    const MAX_OPEN_FILES: u64 = 100;
    pub fn new(dir_path: PathBuf) ->FilePool {
        let mut max_open_files = FilePool::MAX_OPEN_FILES;
        if max_open_files == 0 {
            max_open_files = 100;
        }
        FilePool{
            root_dir_path_buf: dir_path,
            max_open_files,
            files_map: BTreeMap::new(),
            index_queue: VecDeque::new()
        }
    }

    /// Get the handle of the file specified by the index
    ///
    /// return the handle of opened files
    pub fn open(&mut self, index: u64) -> Result<Arc<File>> {
        let result = self.files_map.get(&index);
        match result {
            Some(arc_file) => Ok(arc_file.clone()),
            None => {
                if self.index_queue.len() as u64 >= self.max_open_files {
                    let pop_index = self.index_queue.pop_front().unwrap();
                    self.files_map.remove(&pop_index);
                }
                let new_log_path_buf = self.get_path_buf(index);
                // println!("prepared to open a new file: {}", new_log_path_buf.display());
                let new_file = File::open(new_log_path_buf).map_err(KvStoreError::IOError)?;
                let arc_file = Arc::new(new_file);
                self.files_map.insert(index, arc_file.clone());
                self.index_queue.push_back(index);
                Ok(arc_file)
            },
        }
    }

    /// Get log file's path buf of a given index
    pub fn get_path_buf(&self, index: u64) -> PathBuf {
        self.root_dir_path_buf.join(format!("{}.log", index))
    }

    /// Closed all the cached open files
    pub fn clear(&mut self) {
        self.files_map.clear();
        self.index_queue.clear();
    }
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

impl LogWriteHandle {
    /// Serialize a command and append it to the log file
    fn append_to_log(&mut self, cmd: &Command) -> Result<u64> {
        let serialized = serde_json::to_vec(cmd).map_err(KvStoreError::JSError)?;
        self.file.write_all(&serialized).map_err(KvStoreError::IOError)?;
        // TODO: if append log needs the sync operation?
        // self.file.sync_all().map_err(KvStoreError::IOError)?;
        let len = serialized.len() as u64;
        self.next_pos += len;
        Ok(len)
    }
}

pub struct KvStoreCore {
    /// The root directory
    root_dir_path_buf: PathBuf,
    /// The in-memory index from key to log pointer
    kv_map: HashMap<String, CommandPos>,
    /// The opened file table for get operation
    file_pool: FilePool,
    /// The handle of current writing log file
    log_write_handle: LogWriteHandle,
    /// The hint of compacting log
    repeated_write_count: u64,
}

impl KvStoreCore {
    /// The threshold of trigger compacting log
    const REWRITE_COUNT_THRESH: u64 = 10000;

    /// Judge if the given entry represents a log file (eg: 1.log)
    fn is_log_file(entry: &DirEntry) -> bool {
        let mut flag = false;
        if let Some(s) = entry.file_name().to_str().unwrap_or("").strip_suffix(".log") {
            flag = s.parse::<u64>().is_ok()
        }
        flag
    }

    /// get the root path of database
    fn get_root_path_buf(&self) -> PathBuf {
        self.root_dir_path_buf.clone()
    }

    /// Get log file's path buf of a given index
    fn get_path_buf_by_index(&self, index: u64) -> PathBuf {
        self.root_dir_path_buf.join(format!("{}.log", index))
    }

    /// Get a normal file's path buf of a given name
    pub fn get_path_buf_by_name(&self, name: &str) -> PathBuf {
        self.root_dir_path_buf.join(format!("{}.log", name))
    }

    /// Parse file to Commands, and replay them
    fn read_log_file(kv_map: &mut HashMap<String, CommandPos>, file_pool: &mut FilePool, index: u64) -> Result<()> {
        let file = file_pool.open(index)?;
        let mut stream = Deserializer::from_reader(file.as_ref()).into_iter::<Command>();
        let mut start_pos = 0;
        while let Some(result) = stream.next() {
            let cmd = result.map_err(KvStoreError::JSError)?;
            let next_pos = stream.byte_offset() as u64;
            let cmd_pos = CommandPos{index, start_pos, len: next_pos - start_pos };
            match cmd {
                Command::Get(_) => return Err(KvStoreError::OtherError(format!("The command got from storage is get"))),
                Command::Set(key, _) => kv_map.insert(key, cmd_pos),
                Command::Remove(key) => kv_map.remove(&key),
            };
            start_pos = next_pos;
        }
        Ok(())
    }

    /// Generate Hashmap, FilePool and max file index from log files
    fn load_hashmap_from_log_files(path_buf: &PathBuf) -> Result<(HashMap<String, CommandPos>, FilePool, u64)> {
        if !path_buf.exists() {
            fs::create_dir(path_buf.clone()).map_err(KvStoreError::IOError)?;
        }
        if !path_buf.is_dir() {
            return Err(KvStoreError::OtherError(format!("{} is not dir", path_buf.display())));
        }

        let mut kv_map = HashMap::new();
        let mut file_pool = FilePool::new(path_buf.clone());
        // let log_entries =  path_buf.read_dir().map_err(KvStoreError::IOError)?.filter(KvStore::is_log_file);
        let log_entries =  path_buf.read_dir().map_err(KvStoreError::IOError)?.filter(|x| {
            match x {
                Ok(entry) => KvStoreCore::is_log_file(entry),
                Err(_) => false,
            }
        });

        let mut log_map = BTreeMap::new();
        for item in log_entries {
            let entry = item.unwrap();
            let index = entry.file_name().to_str().unwrap().strip_suffix(".log").unwrap().parse::<u64>().unwrap();
            log_map.insert(index, entry);
        }
        let mut max_index = 0;
        // read log file by index inc order
        for (index, _) in log_map {
            // KvStore::read_log_file(&mut kv_map, &mut file_pool, index).unwrap();
            if let Err(e) = KvStoreCore::read_log_file(&mut kv_map, &mut file_pool, index) {
                panic!("Failed to load from {}.log, detail: {:?}", index, e);
            }
            max_index = index;
        }
        Ok((kv_map, file_pool, max_index))
    }

    /// Return current writing log file's index
    fn index(&self) -> u64 {
        self.log_write_handle.index
    }

    /// Serialize a command and write it to the under storage
    fn append_to_log(&mut self, cmd: &Command) -> Result<u64> {
        self.log_write_handle.append_to_log(cmd)
    }

    /// Open the database
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStoreCore> {
        let root_dir_path_buf = path.into();
        let (kv_map, file_pool, max_index) = KvStoreCore::load_hashmap_from_log_files(&root_dir_path_buf)?;
        let next_index = max_index + 1;
        let new_file = File::create(file_pool.get_path_buf(next_index)).map_err(KvStoreError::IOError)?;

        let core = KvStoreCore{
            root_dir_path_buf,
            kv_map,
            file_pool,
            log_write_handle: LogWriteHandle {
                index: next_index,
                file: new_file,
                next_pos: 0,
            },
            repeated_write_count: 0,
        };
        Ok(core)
    }

    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        match self.kv_map.get(&key) {
            Some(cmd_pos) => {
                match cmd_pos.get_command_with_pool(&mut self.file_pool)? {
                    Command::Set(_, v) => Ok(Some(v)),
                    _ => Err(KvStoreError::OtherError("The got command is not the Set".to_string())),
                }
            },
            None => Ok(None),
        }
    }

    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let start_pos = self.log_write_handle.next_pos;
        let cmd = Command::Set(key.clone(), value.clone());
        let len = self.append_to_log(&cmd).unwrap();
        let prev_val = self.kv_map.insert(key, CommandPos{index: self.index(), start_pos, len});
        if let Some(_) = prev_val {
            self.check_compact_log()?;
        }
        Ok(())
    }

    pub fn remove(&mut self, key: String) -> Result<()> {
        match self.kv_map.remove(&key) {
            Some(_) => {
                let cmd = Command::Remove(key.clone());
                self.append_to_log(&cmd).unwrap();
                self.check_compact_log()?;
                Ok(())
            },
            None => {
                Err(KvStoreError::KeyNotFound)
            },
        }
    }

    /// Compact the log if the repeated_write_count reach the threshold
    fn check_compact_log(&mut self) -> Result<()> {
        self.repeated_write_count += 1;
        if self.repeated_write_count >= KvStoreCore::REWRITE_COUNT_THRESH {
            self.compact_log_sync()?;
            self.repeated_write_count = 0;
        }
        Ok(())
    }

    /// Compact the log
    fn compact_log_sync(&mut self) -> Result<()> {
        let compact_path_buf = self.get_path_buf_by_name("compact.log");
        let new_name_path_buf = self.get_path_buf_by_index(self.index() + 1);
        // Write command to the new log file
        if compact_path_buf.exists() {
            fs::remove_file(compact_path_buf.clone()).map_err(KvStoreError::IOError)?;
        }
        let mut new_log_write_handle = LogWriteHandle {
            index: self.index() + 1,
            file: File::create(compact_path_buf.clone()).map_err(KvStoreError::IOError)?,
            next_pos: 0,
        };

        // copy value to the new_kv_map and compact.log
        let mut new_kv_map = HashMap::with_capacity(self.kv_map.len());
        for (key, cmd_pos)  in self.kv_map.iter() {
            let start_pos = new_log_write_handle.next_pos;
            let cmd = cmd_pos.get_command_with_pool(&mut self.file_pool).unwrap();
            let len = new_log_write_handle.append_to_log(&cmd).unwrap();
            new_kv_map.insert(key.clone(), CommandPos{index: new_log_write_handle.index, start_pos, len});
        }

        // rename compact.log to [num].log
        // println!("compacted log file: {}", new_name_path_buf.display());
        fs::rename(compact_path_buf, new_name_path_buf.clone()).map_err(KvStoreError::IOError)?;
        // update kv_map and log_write_handle
        self.kv_map = new_kv_map;
        self.log_write_handle = new_log_write_handle;

        // delete old log files
        self.file_pool.clear();
        self.drop_old_log_files()
    }

    /// Drop the old log files after compacting operation, called by compact_log_sync()
    fn drop_old_log_files(&mut self) -> Result<()> {
        // let log_entries =  self.root_path_buf.read_dir().expect("read_dir call failed").filter(is_log_file);
        let log_entries =  self.get_root_path_buf().read_dir().map_err(KvStoreError::IOError)?.filter(|x| {
            match x {
                Ok(entry) => KvStoreCore::is_log_file(entry),
                Err(_) => false,
            }
        });
        for item in log_entries {
            let entry = item.map_err(|e| KvStoreError::OtherError(e.to_string()))?;
            let index = entry.file_name().to_str().unwrap().strip_suffix(".log").unwrap().parse::<u64>().unwrap();
            if index < self.index() {
                // println!("drop old file: {}", entry.path().display());
                fs::remove_file(entry.path()).map_err(KvStoreError::IOError)?;
            }
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct KvStore {
    core: Arc<Mutex<KvStoreCore>>,
}

impl KvStore {
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let core = Arc::new(Mutex::new(KvStoreCore::open(path)?));
        Ok(KvStore{core})
    }
}

impl KvsEngine for KvStore {
    fn get(&self, key: String) -> Result<Option<String>> {
        let mut core = self.core.lock().unwrap();
        core.get(key)
    }

    fn set(&self, key: String, value: String) -> Result<()> {
        let mut core = self.core.lock().unwrap();
        core.set(key, value)
    }

    fn remove(&self, key: String) -> Result<()> {
        let mut core = self.core.lock().unwrap();
        core.remove(key)
    }

    fn get_name(&self) -> String {
        KVSTORE_ENGINE_NAME.to_string()
    }
}