#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use std::thread;
use std::process;
use std::thread::JoinHandle;
use lua::ternary;
use std::path::PathBuf;
use std::time::Duration;
use std::collections::HashMap;
use std::sync::{ Arc, Mutex };
use std::fs:: { self, OpenOptions };
use memmap2::{ MmapOptions, MmapMut };
use chrono::{ DateTime, Datelike, Timelike, Utc };

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
    time: DateTime<Utc>,
    mapbuf: Option<MmapMut>,
}

impl FileDest {
    pub fn new(path: PathBuf, feature: &str, sz: usize, rtype: u32) -> FileDest {
        FileDest {
            size: 0,
            mapbuf: None,
            max_size: sz,
            alloc_size: 0,
            time: Utc::now(),
            rolling_type: rtype,
            feature: feature.to_string(),
            file_path: PathBuf::from(""),
            clean_time: CLEAN_TIME,
            ignore_prefix: false,
            ignore_suffix: true,
            log_path: path,
        }
    }

    fn check_full(&self) -> bool {
        return self.alloc_size > self.max_size;
    }

    fn new_log_file_name(&self, log: &LogMessage) ->String {
        format!("{}-{}.{:03}.p{}.log", self.feature, log.time.format("%Y-%m-%d-%H%M%S"), log.time.timestamp_subsec_millis(), process::id())
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
        self.time = Utc::now();
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
    pub time: DateTime<Utc>,
}

impl LogMessage {
    pub fn new() -> LogMessage {
        LogMessage {
            line: 0,
            level: 0,
            grow: false,
            time: Utc::now(),
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
            self.time = Utc::now();
            self.source = source.to_string();
    }

    pub fn set_grow(&mut self, grow: bool) {
        self.grow = grow;
    }
}

type MessageList = Vec<Arc<Mutex<LogMessage>>>;

struct LogMessagePool {
    free_messages: Arc<Mutex<MessageList>>,
    alloc_messages: Arc<Mutex<MessageList>>,
}

impl LogMessagePool  {
    pub fn new() -> LogMessagePool  {
        LogMessagePool {
            free_messages: Arc::new(Mutex::new(Vec::with_capacity(QUEUE_SIZE))),
            alloc_messages: Arc::new(Mutex::new(Vec::with_capacity(QUEUE_SIZE))),
        }
    }

    pub fn init(&mut self) {
        for _ in 0..QUEUE_SIZE {
            self.alloc_messages.lock().unwrap().push(Arc::new(Mutex::new(LogMessage::new())));
        }
    }

    pub fn allocate(&mut self) -> Arc<Mutex<LogMessage>> {
        let mut alloc_vec = self.alloc_messages.lock().unwrap();
        if !alloc_vec.is_empty() {
            return alloc_vec.pop().unwrap();
        }
        let mut free_vec = self.free_messages.lock().unwrap();
        if free_vec.is_empty() {
            let mut msg = LogMessage::new();
            msg.set_grow(true);
            return Arc::new(Mutex::new(msg));
        }
        std::mem::swap(&mut *free_vec, &mut *alloc_vec);
        alloc_vec.pop().unwrap()
    }

    pub fn release(&mut self, msg : Arc<Mutex<LogMessage>>) {
        if !msg.lock().unwrap().grow {
            let mut free_vec = self.free_messages.lock().unwrap();
            free_vec.push(msg);
        }
    }
}

struct LogMessageQueue {
    read_messages: Arc<Mutex<MessageList>>,
    write_messages: Arc<Mutex<MessageList>>,
}

impl LogMessageQueue  {
    pub fn new() -> LogMessageQueue  {
        LogMessageQueue {
            read_messages: Arc::new(Mutex::new(Vec::new())),
            write_messages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn put(&mut self, logmsg: Arc<Mutex<LogMessage>>) {
        let mut write_vec = self.write_messages.lock().unwrap();
        write_vec.push(logmsg);
    }

    pub fn timed_getv(&mut self) -> Arc<Mutex<MessageList>> {
        {
            let mut read_vec = self.read_messages.lock().unwrap();
            let mut write_vec  = self.write_messages.lock().unwrap();
            read_vec.clear();
            std::mem::swap(&mut *read_vec, &mut *write_vec);
        }
        let read_vec = self.read_messages.lock().unwrap();
        if read_vec.is_empty() {
            thread::sleep(Duration::from_millis(5));
        }
        self.read_messages.clone()
    }
}

pub struct LogService {
    path: String,
    service: String,
    rolling_type: u32,
    logmsgque: LogMessageQueue,
    messagepool: LogMessagePool,
    stop_msg: Arc<Mutex<LogMessage>>,
    dest_lvls: Mutex<HashMap<u32, Arc<Mutex<FileDest>>>>,
    dest_features: Mutex<HashMap<String, Arc<Mutex<FileDest>>>>,
    def_dest: Option<Arc<Mutex<FileDest>>>,
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
            logmsgque: LogMessageQueue::new(),
            messagepool: LogMessagePool::new(),
            dest_lvls: Mutex::new(HashMap::new()),
            dest_features: Mutex::new(HashMap::new()),
            stop_msg: Arc::new(Mutex::new(LogMessage::new())),
            rolling_type: RollingType::DAYLY as u32,
            clean_time: CLEAN_TIME,
            thread_handle: None,
            max_size: MAX_SIZE,
            def_dest: None,
            filter_bits: -1,
            daemon: false,
        }
    }

    pub fn start(&mut self) {
    }

    pub fn stop(&mut self) {
        self.logmsgque.put(self.stop_msg.clone());
        if self.thread_handle.is_some() {
            self.thread_handle.take().unwrap().join().unwrap();
        }
    }

    pub fn option(&mut self, logger: Arc<Mutex<LogService>>, path: String, service: String, index: String) {
        self.path = path;
        self.messagepool.init();
        self.service = format!("{}-{}", service, index);
        if self.thread_handle.is_none() {
            self.thread_handle = Some(thread::spawn(|| {
                LogService::run(logger);
            }));
        }
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
        let dest_features = self.dest_features.lock().unwrap();
        let dest = dest_features.get(&feature);
        if dest.is_some() {
            let dest = dest.unwrap();
            dest.lock().unwrap().set_clean_time(clean_time);
        }
    }

    pub fn ignore_prefix(&mut self, feature: String, prefix: bool) {
        let dest_features = self.dest_features.lock().unwrap();
        let dest = dest_features.get(&feature);
        if dest.is_some() {
            let dest = dest.unwrap();
            dest.lock().unwrap().ignore_prefix(prefix);
        }
    }

    pub fn ignore_suffix(&mut self, feature: String, suffix: bool) {
        let dest_features = self.dest_features.lock().unwrap();
        let dest = dest_features.get(&feature);
        if dest.is_some() {
            let dest = dest.unwrap();
            dest.lock().unwrap().ignore_suffix(suffix);
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
        let mut dest_features = self.dest_features.lock().unwrap();
        if !dest_features.contains_key(&feature) {
            let path = self.build_path(&feature);
            let dest = Arc::new(Mutex::new(FileDest::new(path, &feature, self.max_size, self.rolling_type)));
            dest_features.insert(feature.clone(), dest.clone());
            if self.def_dest.is_none() {
                self.def_dest = Some(dest);
            }
        }
    }

    pub fn add_file_dest(&mut self, feature: String, fname: String) {
        let mut dest_features = self.dest_features.lock().unwrap();
        if !dest_features.contains_key(&feature) {
            let path = self.build_path(&self.service);
            let _ = fs::create_dir_all(&path);
            let mut filedest = FileDest::new(path, &feature, self.max_size, RollingType::NONE as u32);
            filedest.create(fname);
            filedest.ignore_prefix(true);
            dest_features.insert(feature.clone(), Arc::new(Mutex::new(filedest)));
        }
    }
    
    pub fn add_lvl_dest(&mut self, level: u32) {
        let mut dest_lvls = self.dest_lvls.lock().unwrap();
        if !dest_lvls.contains_key(&level) {
            let mut feature = LEVEL_NAMES[level as usize].to_string();
            feature.make_ascii_lowercase();
            let mut path = self.build_path(&self.service);
            path.push(&feature);
            let dest = Arc::new(Mutex::new(FileDest::new(path, &feature, self.max_size, self.rolling_type)));
            dest_lvls.insert(level, dest.clone());
        }
    }

    pub fn del_dest(&mut self, feature: String) {
        let mut dest_features = self.dest_features.lock().unwrap();
        if dest_features.contains_key(&feature) {
            dest_features.remove(&feature);
        }
    }

    pub fn del_lvl_dest(&mut self, log_lvl: u32) {
        let mut dest_lvls = self.dest_lvls.lock().unwrap();
        if dest_lvls.contains_key(&log_lvl) {
            dest_lvls.remove(&log_lvl);
        }
    }

    pub fn output(&mut self, level: u32, msg: &String, tag: String, feature: String, source: &str, line: i32) {
        if !self.is_filter(level) {
            let message = self.messagepool.allocate();
            message.lock().unwrap().option(level, msg.to_string(), tag, feature, source, line);
            self.logmsgque.put(message.clone());
        }
    }

    pub fn run(log_service: Arc<Mutex<LogService>>) {
        let mut running = true;
        thread::sleep(Duration::from_millis(100));
        while running {
            let mut logger = log_service.lock().unwrap();
            let list = logger.logmsgque.timed_getv();
            let logmsgs = list.lock().unwrap();
            for logmsg in logmsgs.iter() {
                let log = logmsg.lock().unwrap();
                if log.level == 0 {
                    running = false;
                    continue;
                }
                if !logger.daemon {
                    logger.std_dest.write(&log);
                }
                let dest_lvls = logger.dest_lvls.lock().unwrap();
                let lv_dest = dest_lvls.get(&log.level);
                if lv_dest.is_some() {
                    lv_dest.unwrap().lock().unwrap().write(&log);
                }
                let dest_features = logger.dest_features.lock().unwrap();
                let fea_dest = dest_features.get(&log.feature);
                if fea_dest.is_some() {
                    fea_dest.unwrap().lock().unwrap().write(&log);
                } else {
                    let def_dest = logger.def_dest.clone();
                    def_dest.unwrap().lock().unwrap().write(&log);
                }
            }
        }
    }
}
