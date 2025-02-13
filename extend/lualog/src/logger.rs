#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use std::thread;
use std::process;
use lua::ternary;
use dashmap::DashMap;
use std::path::PathBuf;
use std::time::Duration;
use std::thread::JoinHandle;
use std::sync::{ Arc, Mutex };
use std::fs::{ self, OpenOptions };
use memmap2::{ MmapOptions, MmapMut };
use chrono::{ Local, DateTime, Datelike, Timelike };

const QUEUE_SIZE: usize = 10000;
const PAGE_SIZE: usize  = 65536;
const MAX_SIZE: usize   = 1024 * 1024 * 16;
const CLEAN_TIME: u64   = 7 * 24 * 3600;

const LEVEL_NAMES: [&str; 7] = ["UNKNW", "DEBUG", "INFO", "WARN", "DUMP", "ERROR", "FATAL"];
const LEVEL_COLORS: [&str; 7] = ["\x1b[32m", "\x1b[37m", "\x1b[32m", "\x1b[33m", "\x1b[33m", "\x1b[31m", "\x1b[32m"];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RollingType {
    NONE,
    DAYLY,
    HOURLY,
}

pub enum LogLevel {
    DEBUG = 1,
    INFO,
    WARN,
    DUMP,
    ERROR,
    FATAL,
}

trait LogDest {
    fn build_prefix(&mut self, log: &LogMessage, iprefix: bool) -> String {
        if !iprefix {
            return format!("[{}.{:03}][{}]", log.time.format("%Y-%m-%d %H:%M:%S"), log.time.timestamp_subsec_millis(), LEVEL_NAMES[log.level as usize]);
        }
        "".to_string()
    }
    fn build_suffix(&mut self, log: &LogMessage, isuffix: bool) -> String {
        ternary!(isuffix, "".to_string(), format!("[{}:{}] ", log.source, log.line))
    }
    fn ignore_prefix(&mut self, prefix: bool){}
    fn ignore_suffix(&mut self, suffix: bool){}
    fn set_clean_time(&mut self, clean_time: u64){}
    fn raw_write(&mut self, msg: String, lvl: u32) {
        println!("{}{}", LEVEL_COLORS[lvl as usize], msg);
    }
    fn write(&mut self, log: &LogMessage);
}

struct StdDest;
impl LogDest for StdDest {
    fn write(&mut self, log: &LogMessage) {
        let logtxt = format!("{} {}{}", self.build_prefix(log, false), log.msg, self.build_suffix(log, true));
        self.raw_write(logtxt, log.level);
    }
}

struct FileDest {
    size: usize,
    max_size: usize,
    alloc_size: usize,
    clean_time: u64,
    feature: String,
    log_path: PathBuf,
    file_path: PathBuf,
    rolling_type: u32,
    ignore_suffix: bool,
    ignore_prefix: bool,
    time: DateTime<Local>,
    mapbuf: Option<MmapMut>,
}

impl FileDest {
    pub fn new() -> FileDest {
        FileDest {
            size: 0,
            mapbuf: None,
            alloc_size: 0,
            time: Local::now(),
            max_size: MAX_SIZE,
            clean_time: CLEAN_TIME,
            rolling_type: RollingType::DAYLY as u32,
            file_path: PathBuf::from(""),
            log_path: PathBuf::from(""),
            feature: "".to_string(),
            ignore_prefix: false,
            ignore_suffix: true,
        }
    }

    fn setup(&mut self, path: PathBuf, feature: &str, sz: usize, rtype: u32, clean_time: u64) {
        self.max_size = sz;
        self.log_path = path;
        self.rolling_type = rtype;
        self.clean_time = clean_time;
        self.feature = feature.to_string();
    }

    fn check_full(&self) -> bool {
        return self.alloc_size > self.max_size;
    }

    fn new_log_file_name(&self, log: &LogMessage) ->String {
        format!("{}-{}.{:03}.p{}.log", self.feature, log.time.format("%Y%m%d-%H%M%S"), log.time.timestamp_subsec_millis(), process::id())
    }

    fn unmap_file(&mut self,){
        self.mapbuf = None;
    }

    fn map_file(&mut self,){
        let res = OpenOptions::new().read(true).write(true).create(true).open(&self.file_path);
        match res {
            Ok(file) => {
                file.set_len(self.alloc_size as u64).expect("Failed to set file size");
                let mres = unsafe { MmapOptions::new().map_mut(&file) };
                match mres {
                    Ok(map) => {self.mapbuf = Some(map)},
                    Err(e) => {
                        println!("Failed to map file: {}, err: {}", self.file_path.to_str().unwrap(), e);
                    }
                }
            },
            Err(e) => {
                println!("Failed to open file: {}, err: {}", self.file_path.to_str().unwrap(), e);
            }
        }
    }

    fn rolling_eval(&self, log: &LogMessage) ->bool {
        if self.rolling_type == RollingType::NONE as u32 {
            return false;
        }
        let ltime = log.time;
        let ftime = self.time;
        if self.rolling_type == RollingType::DAYLY as u32 {
            return ftime.day() != ltime.day() && ftime.month() != ltime.month() && ftime.year() != ltime.year();
        }
        return ftime.hour() != ltime.hour() && ftime.day() != ltime.day() && ftime.month() != ltime.month() && ftime.year() != ltime.year();
    }
    
    pub fn create(&mut self, file_name: String) {
        self.unmap_file();
        self.size = 0;
        self.time = Local::now();
        self.alloc_size = PAGE_SIZE;
        self.file_path = self.log_path.join(file_name);
        self.map_file();
    }
}

impl LogDest for FileDest {
    fn ignore_prefix(&mut self, prefix: bool){
        self.ignore_prefix = prefix;
    }
    fn ignore_suffix(&mut self, suffix: bool){
        self.ignore_suffix = suffix;
    }
    fn set_clean_time(&mut self, clean_time: u64){
        self.clean_time = clean_time;
    }
    fn raw_write(&mut self, msg: String, lvl: u32) {
        if self.size + msg.len() > self.alloc_size {
            self.alloc_size += PAGE_SIZE;
            self.unmap_file();
            self.map_file();
        }
        if self.mapbuf.is_some() {
            let end = self.size + msg.len();
            let map = self.mapbuf.as_mut().unwrap();
            map[self.size..end].copy_from_slice(msg.as_bytes());
            self.size = end;
        }
    }
    fn write(&mut self, log: &LogMessage) {
        let logtxt: String = format!("{} {}{}\n", self.build_prefix(log, self.ignore_prefix), log.msg, self.build_suffix(log, self.ignore_suffix));
        if self.mapbuf.is_none() || self.check_full() || self.rolling_eval(log) {
            let _ = fs::create_dir_all(&self.log_path);
            self.create(self.new_log_file_name(log));
        }
        self.raw_write(logtxt, log.level);
    }
}

struct LogMessage {
    pub line: i32,
    pub level: u32,
    pub grow: bool,
    pub tag: String,
    pub msg: String,
    pub source: String,
    pub feature: String,
    pub time: DateTime<Local>,
}

impl LogMessage {
    pub fn new(grow: bool) -> LogMessage {
        LogMessage {
            line: 0,
            level: 0,
            grow: grow,
            time: Local::now(),
            tag: "".to_string(),
            msg: "".to_string(),
            source: "".to_string(),
            feature: "".to_string(),
        }
    }
    
    pub fn option(&mut self, level: u32, msg: String, tag: String, feature: String, source: &str, line: i32) {
            self.line = line;
            self.tag = tag;
            self.msg = msg;
            self.level = level;
            self.feature = feature;
            self.time = Local::now();
            self.source = source.to_string();
    }

    pub fn set_grow(&mut self, grow: bool) {
        self.grow = grow;
    }
}

struct LogMessagePool {
    mutex: Mutex<()>,
    free_messages: Vec<LogMessage>,
    alloc_messages: Vec<LogMessage>,
}

impl LogMessagePool  {
    pub fn new() -> LogMessagePool  {
        LogMessagePool {
            mutex: Mutex::new(()),
            free_messages: Vec::new(),
            alloc_messages: Vec::new(),
        }
    }

    pub fn init(&mut self) {
        for _ in 0..QUEUE_SIZE {
            self.alloc_messages.push(LogMessage::new(false));
        }
    }

    pub fn allocate(&mut self) -> LogMessage {
        if self.alloc_messages.is_empty() {
            let _g = self.mutex.lock();
            std::mem::swap(&mut self.alloc_messages, &mut self.free_messages);
        }
        if self.alloc_messages.is_empty() {
            return LogMessage::new(true);
        }
        let _g = self.mutex.lock();
        self.alloc_messages.pop().unwrap()
    }

    pub fn release(&mut self, msg : LogMessage) {
        if !msg.grow {
            let _g = self.mutex.lock();
            self.free_messages.push(msg);
        }
    }
}

type MessageList = Vec<LogMessage>;
struct LogMessageQueue {
    mutex: Mutex<()>,
    read_messages: Vec<LogMessage>,
    write_messages: Vec<LogMessage>,
}

impl LogMessageQueue  {
    pub fn new() -> LogMessageQueue  {
        LogMessageQueue {
            mutex: Mutex::new(()),
            read_messages: Vec::new(),
            write_messages: Vec::new(),
        }
    }

    pub fn put(&mut self, logmsg: LogMessage) {
        let _g = self.mutex.lock();
        self.write_messages.push(logmsg);
    }

    pub fn timed_getv(&mut self) ->&Vec<LogMessage> {
        {
            self.read_messages.clear();
            let _g = self.mutex.lock();
            std::mem::swap(&mut self.read_messages, &mut self.write_messages);
        }
        if self.read_messages.is_empty() {
            thread::sleep(Duration::from_millis(5));
        }
        &self.read_messages
    }
}

pub struct LogService {
    path: String,
    service: String,
    rolling_type: u32,
    main_dest: FileDest,
    logmsgque: LogMessageQueue,
    messagepool: LogMessagePool,
    dest_lvls: DashMap<u32, FileDest>,
    dest_features: DashMap<String, FileDest>,
    thread_handle: Option<JoinHandle<()>>,
    std_dest: StdDest,
    filter_bits: i32,
    clean_time: u64,
    max_size: usize,
    daemon: bool,
}

impl LogService {
    pub fn new() -> LogService {
        LogService {
            std_dest: StdDest,
            path: "".to_string(),
            service: "".to_string(),
            dest_lvls: DashMap::new(),
            dest_features: DashMap::new(),
            logmsgque: LogMessageQueue::new(),
            messagepool: LogMessagePool::new(),
            rolling_type: RollingType::DAYLY as u32,
            main_dest: FileDest::new(),
            clean_time: CLEAN_TIME,
            thread_handle: None,
            max_size: MAX_SIZE,
            filter_bits: -1,
            daemon: false,
        }
    }

    pub fn start(&mut self) {
        self.messagepool.init();
    }

    pub fn stop(&mut self) {
        self.logmsgque.put(LogMessage::new(false));
        if self.thread_handle.is_some() {
            self.thread_handle.take().unwrap().join().unwrap();
        }
    }

    pub fn option(&mut self, logger: Arc<Mutex<LogService>>, path: String, service: String, index: String) {
        self.path = path.clone();
        self.add_main_dest(&service, &index);
        if self.thread_handle.is_none() {
            self.thread_handle = Some(thread::spawn(|| {
                LogService::run(logger);
            }));
        }
    }

    pub fn add_main_dest(&mut self, service: &str, index: &str) {
        let _ = fs::create_dir_all(&self.path);
        self.service = format!("{}-{}", service, index);
        let logpath = self.build_path(&self.service);
        self.main_dest.setup(logpath, service, self.max_size, self.rolling_type, self.clean_time);
    }

    pub fn daemon(&mut self, status: bool) {
        self.daemon = status;
    }

    pub fn set_max_size(&mut self, size: usize,) {
        self.max_size = size;
    }

    pub fn set_clean_time(&mut self, time: u64) {
        self.clean_time = time;
    }
    
    pub fn set_rolling_type(&mut self, rt: u32) {
        self.rolling_type = rt;
    }

    pub fn filter(&mut self, lv: u32, on: bool) {
        match on {
            true => self.filter_bits |= 1 << (lv - 1),
            false => self.filter_bits &= !(1 << (lv - 1)),
        }
    }

    pub fn is_filter(&self, lv: u32) -> bool {
        return 0 == (self.filter_bits & (1 << (lv - 1))); 
    }

    pub fn set_dest_clean_time(&mut self, feature: String, clean_time: u64) {
        if let Some(mut dest) = self.dest_features.get_mut(&feature) {
           dest.set_clean_time(clean_time)
        }
    }

    pub fn ignore_prefix(&mut self, feature: String, prefix: bool) {
        if let Some(mut dest) = self.dest_features.get_mut(&feature) {
           dest.ignore_prefix(prefix);
        }
    }

    pub fn ignore_suffix(&mut self, feature: String, suffix: bool) {
        if let Some(mut dest) = self.dest_features.get_mut(&feature) {
            dest.ignore_suffix(suffix);
        }
    }

    pub fn build_path(&self, feature: &str) -> PathBuf  {
        let mut path = PathBuf::from(&self.path);
        if self.service == feature {
            path.push(&self.service)
        } else {
            path.push(feature)
        }
        path
    }
    
    pub fn add_dest(&mut self, feature: String) {
        if !self.dest_features.contains_key(&feature) {
            let mut dest = FileDest::new();
            let path = self.build_path(&feature);
            dest.setup(path, &feature, self.max_size, self.rolling_type, self.clean_time);
            self.dest_features.insert(feature, dest);
        }
    }

    pub fn add_file_dest(&mut self, feature: String, fname: String) {
        if !self.dest_features.contains_key(&feature) {
            let path = self.build_path(&self.service);
            let _ = fs::create_dir_all(&path);
            let mut dest = FileDest::new();
            dest.setup(path, &feature, self.max_size, RollingType::NONE as u32, self.clean_time);
            dest.create(fname);
            dest.ignore_prefix(true);
            self.dest_features.insert(feature.clone(), dest);
        }
    }
    
    pub fn add_lvl_dest(&mut self, level: u32) {
        if !self.dest_lvls.contains_key(&level) {
            let mut feature = LEVEL_NAMES[level as usize].to_string();
            feature.make_ascii_lowercase();
            let mut path = self.build_path(&self.service);
            path.push(&feature);
            let mut dest = FileDest::new();
            dest.setup(path, &feature, self.max_size, self.rolling_type, self.clean_time);
            self.dest_lvls.insert(level, dest);
        }
    }

    pub fn del_dest(&mut self, feature: String) {
        self.dest_features.remove(&feature);
    }

    pub fn del_lvl_dest(&mut self, log_lvl: u32) {
        self.dest_lvls.remove(&log_lvl);
    }

    pub fn output(&mut self, level: u32, msg: &String, tag: String, feature: String, source: &str, line: i32) {
        if !self.is_filter(level) {
            let mut message = self.messagepool.allocate();
            message.option(level, msg.to_string(), tag, feature, source, line);
            self.logmsgque.put(message);
        }
    }

    pub fn update(&mut self) -> bool {
        let logmsgs = self.logmsgque.timed_getv();
        for log in logmsgs.iter() {
            if log.level == 0 {
                return false
            }
            if !self.daemon {
                self.std_dest.write(&log);
            }
            if let Some(mut dest) = self.dest_lvls.get_mut(&log.level) {
                dest.write(&log);
            }
            if let Some(mut dest) = self.dest_features.get_mut(&log.feature) {
                dest.write(&log);
            } else {
                self.main_dest.write(&log);
            }
        }
        true
    }

    pub fn run(log_service: Arc<Mutex<LogService>>) {
        thread::sleep(Duration::from_millis(100));
        loop {
            if let Ok(mut logger) = log_service.lock() {
                if !logger.update() {
                    break;
                }
            }
        }
    }
}
