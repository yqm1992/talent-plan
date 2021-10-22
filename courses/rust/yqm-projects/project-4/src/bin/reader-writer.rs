use kvs::{KvStoreError, Result, Command};
use std::net::TcpListener;
use std::sync::{Mutex, RwLock};
use std::cell::{Cell, RefCell};
use std::borrow::{Borrow, BorrowMut};
use std::thread;

use kvs::{ThreadPool, NaiveThreadPool};
use std::collections::{HashMap, BTreeMap};
use std::fs::{File, DirEntry};
use std::path::PathBuf;
use std::str::FromStr;
use std::io::{Read, BufReader, Seek, SeekFrom, Write, BufWriter};
use std::fmt::Arguments;
use serde_json::value::Serializer;
use serde::Serialize;
use std::path::Component::CurDir;


fn ttttt() {
    let a = Some(4);
}

#[allow(dead_code)]
fn test() {
    let addr = "127.0.0.1:4004";
    let mut listener_opt: Option<TcpListener> = None;
    for i in 0..4 {
        match TcpListener::bind(addr.clone()) {
            Ok(t) => {
                listener_opt = Some(t);
                break;
            }
            Err(e) => {
                if i >= 3 {
                    break;
                }
                println!("Failed to create listener at addr {:?}, {:?}, sleep = {}s, retry...", addr, e, (i+1)*2);
                std::thread::sleep(std::time::Duration::from_secs((i+1)*2));
            },
        }
    }
    match listener_opt {
        Some(_) => println!("Success to create listener"),
        None => println!("Failed to create listener, give up"),
    }
}


fn test_reader() {
    let path_buf = PathBuf::from_str(".").unwrap();
    let (readers, kv_map, max_index) = load(path_buf).unwrap();
}

fn test_writer(index: u64) {
    let path = format!("{}.log", index);
    let file = std::fs::OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(path)
        .unwrap();
    let mut writer = BufWriterWithPos::new(file).unwrap();
    for i in 90..100 {
        let cmd = Command::Set(i.to_string(), i.to_string());
        let (start_pos , len) = writer.write_cmd(cmd);
        let cmd_pos = CommandPos{index, start_pos, len};
    }
}

fn main() {
    // let index = 111;
    // test_writer(index);
    // test_reader(index);
    test_reader();
}

fn is_log_file(entry: &DirEntry) -> bool {
    let mut flag = false;
    if let Some(s) = entry.file_name().to_str().unwrap_or("").strip_suffix(".log") {
        flag = s.parse::<u64>().is_ok()
    }
    flag
}

fn scan_log_files(path_buf: PathBuf) -> Result<BTreeMap<u64, DirEntry>> {
    if !path_buf.exists() {
        std::fs::create_dir(path_buf.clone()).map_err(KvStoreError::IOError)?;
    }
    if !path_buf.is_dir() {
        return Err(KvStoreError::OtherError(format!("{} is not dir", path_buf.display())));
    }
    let mut log_map = BTreeMap::new();
    let log_entries =  path_buf.read_dir().map_err(KvStoreError::IOError)?.filter(|x| {
        match x {
            Ok(entry) => is_log_file(entry),
            Err(_) => false,
        }
    });
    for item in log_entries {
        let entry = item.unwrap();
        let index = entry.file_name().to_str().unwrap().strip_suffix(".log").unwrap().parse::<u64>().map_err(KvStoreError::ParseError)?;
        log_map.insert(index, entry);
    }
    Ok(log_map)
}

fn load(path_buf: PathBuf) -> Result<(HashMap<u64, BufReaderWithPos<File>>, HashMap<String, CommandPos>, u64)> {
    
    let mut readers = HashMap::new();
    let mut kv_map = HashMap::new();
    let mut log_map = scan_log_files(path_buf.clone())?;
    let mut max_index = 0;
    // read log file by index inc order
    for (index, entry) in log_map {
        let file = std::fs::File::open(entry.path()).map_err(KvStoreError::IOError)?;
        println!("load {}", entry.path().display());
        load_single_file(index, file, &mut readers, &mut kv_map)?;
        max_index = index;
    }
    let ret = (readers, kv_map, max_index);
    Ok(ret)
}

fn load_single_file(index: u64, file: File, readers: &mut HashMap<u64, BufReaderWithPos<File>>, kv_map: &mut HashMap<String, CommandPos>) -> Result<()> {
    let mut reader = BufReaderWithPos::new(file)?;
    let mut de = serde_json::Deserializer::from_reader(&mut reader).into_iter::<Command>();
    let mut start_pos = 0;
    while let Some(item) = de.next() {
        let next_pos = de.byte_offset() as u64;
        let len = next_pos - start_pos;
        let cmd = item.unwrap();
        let cmd_pos = CommandPos{index, start_pos, len};
        // println!("{:?}, {:?}", cmd_pos, cmd);
        match cmd {
            Command::Set(key, val) => {kv_map.insert(key, cmd_pos); },
            Command::Remove(key) => {kv_map.remove(&key);},
            _ => {},
        }
        start_pos = next_pos;
    }
    readers.insert(index, reader);
    Ok(())
}

#[derive(Debug)]
struct CommandPos {
    index: u64,
    start_pos: u64,
    len: u64,
}


struct Store {
    readers: HashMap<u64, BufReaderWithPos<File>>,
    writer: BufWriterWithPos<File>,
    index_map: HashMap<String, CommandPos>,
    cur_index: u64,
}

impl Store {
    pub fn get(&mut self, key: String) -> Result<Option<Command>> {
        match self.index_map.get(&key) {
            Some(cmd_pos) => {
                let mut reader = self.readers.get_mut(&cmd_pos.index).expect("Can not find log file");
                reader.seek(SeekFrom::Start(cmd_pos.start_pos)).map_err(KvStoreError::IOError)?;
                let mut cmd_reader = reader.take(cmd_pos.len);
                let mut de = serde_json::Deserializer::from_reader(cmd_reader).into_iter::<Command>();
                if let Some(Ok(Command::Set(k, v))) = de.next() {
                    Ok(Some(Command::Set(k, v)))
                } else {
                    Err(KvStoreError::OtherError("unexpected cmd type".to_string()))
                }
            },
            None => Ok(None),
        }
    }
}

struct BufReaderWithPos<T: Read + Seek> {
    reader: BufReader<T>,
    pos: u64,
}

impl<T: Read + Seek> BufReaderWithPos<T> {
    pub fn new(mut t: T) -> Result<Self> {
        let mut reader = BufReader::new(t);
        reader.seek(SeekFrom::Start(0)).map_err(KvStoreError::IOError)?;
        let pos = 0_u64;
        Ok(BufReaderWithPos{reader, pos})
    }
    pub fn get_pos(&self) -> u64 {
        self.pos
    }
}

impl<T: Read + Seek> Read for BufReaderWithPos<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.reader.read(buf)?;
        self.pos += n as u64;
        Ok(n)
    }
}

impl<T: Read + Seek> Seek for BufReaderWithPos<T> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.pos = self.reader.seek(pos)?;
        Ok(self.pos)
    }
}

struct BufWriterWithPos<T: Write + Seek> {
    writer: BufWriter<T>,
    pos: u64,
}

impl<T: Write + Seek> BufWriterWithPos<T> {
    pub fn new(mut t: T) -> Result<Self> {
        let mut writer = BufWriter::new(t);
        writer.seek(SeekFrom::Start(0)).map_err(KvStoreError::IOError)?;
        let pos = 0_u64;
        Ok(BufWriterWithPos{writer, pos})
    }
    pub fn get_pos(&self) -> u64 {
        self.pos
    }
    pub fn write_cmd(&mut self, cmd: Command) -> (u64, u64) {
        let start_pos = self.get_pos();
        let vec = serde_json::to_vec(&cmd).unwrap();
        self.write_all(&vec).unwrap();
        (start_pos, vec.len() as u64)
    }
}

impl<T: Write + Seek> Write for BufWriterWithPos<T> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let len = self.writer.write(buf)?;
        self.pos += len as u64;
        Ok(len)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

impl<T: Write + Seek> Seek for BufWriterWithPos<T> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.pos = self.writer.seek(pos)?;
        Ok(self.pos)
    }
}