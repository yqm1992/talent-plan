use std::collections::{HashMap, BTreeMap, VecDeque};
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use serde_json::Deserializer;
use std::fs;
use std::fs::{File, DirEntry};
use std::io::{Write, Seek, SeekFrom};
use std::rc::Rc;


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

impl CommandPos {
    /// Read command from the log file
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
    files_map: BTreeMap<u64, Rc<File>>,
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

    /// Open index log file if is is not opened
    /// and return it
    pub fn open(&mut self, index: u64) -> Result<Rc<File>> {
        let result = self.files_map.get(&index);
        match result {
            Some(rc_file) => Ok(rc_file.clone()),
            None => {
                if self.index_queue.len() as u64 >= self.max_open_files {
                    let pop_index = self.index_queue.pop_front().unwrap();
                    self.files_map.remove(&pop_index);
                }
                let new_log_path_buf = self.get_path_buf(index);
                // println!("prepared to open a new file: {}", new_log_path_buf.display());
                let new_file = File::open(new_log_path_buf).map_err(KvStoreError::IOError)?;
                let rc_file = Rc::new(new_file);
                self.files_map.insert(index, rc_file.clone());
                Ok(rc_file)
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

pub struct KvStore {
    /// The root directory
    root_dir_path_buf: PathBuf,
    /// The in-memory index from key to log pointer
    kv_map: HashMap<String, CommandPos>,
    /// The opened file table for get operation
    file_pool: FilePool,
    /// The handle of current writing log file
    log_write_handle: LogWriteHandle,
}

impl KvStore {
    /// Judge if the given entry represents a log file (eg: 1.log)
    fn is_log_file(entry: &DirEntry) -> bool {
        let mut flag = false;
        if let Some(s) = entry.file_name().to_str().unwrap_or("").strip_suffix(".log") {
            flag = s.parse::<u64>().is_ok()
        }
        flag
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
                Ok(entry) => KvStore::is_log_file(entry),
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
            if let Err(e) = KvStore::read_log_file(&mut kv_map, &mut file_pool, index) {
                panic!("Failed to load from {}.log, detail: {:?}", index, e);
            }
            max_index = index;
        }
        Ok((kv_map, file_pool, max_index))
    }

    /// Return current writing log file's index
    pub fn index(&self) -> u64 {
        self.log_write_handle.index
    }

    /// Serialize a command and write it to the under storage
    fn append_to_log(&mut self, cmd: &Command) -> Result<u64> {
        self.log_write_handle.append_to_log(cmd)
    }

    /// Open the database
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let root_dir_path_buf = path.into();
        let (kv_map, file_pool, max_index) = KvStore::load_hashmap_from_log_files(&root_dir_path_buf)?;
        let next_index = max_index + 1;
        let new_file = File::create(file_pool.get_path_buf(next_index)).map_err(KvStoreError::IOError)?;

        let kv_store = KvStore{
            root_dir_path_buf,
            kv_map,
            file_pool,
            log_write_handle: LogWriteHandle {
                index: next_index,
                file: new_file,
                next_pos: 0,
            },
        };
        Ok(kv_store)
    }
    /// Get the value of key from KvStore
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        match self.kv_map.get(&key) {
            Some(cmd_pos) => {
                match cmd_pos.get_command_with_pool(&mut self.file_pool)? {
                    Command::Set(_, v) => Ok(Some(v)),
                    // Error
                    // _ => panic!("The command is not the Set Command"),
                    _ => Err(KvStoreError::OtherError("The got command is not the Set".to_string())),
                }
            },
            None => Ok(None),
        }
    }
    /// Set value for key in KvStore
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let start_pos = self.log_write_handle.next_pos;
        let cmd = Command::Set(key.clone(), value.clone());
        let len = self.append_to_log(&cmd).unwrap();
        self.kv_map.insert(key, CommandPos{index: self.index(), start_pos, len});
        Ok(())
    }
    /// Remove a key from KvStore
    pub fn remove(&mut self, key: String) -> Result<()> {
        Err(KvStoreError::OtherError("no implementation".to_string()))
    }
}
