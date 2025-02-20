#![allow(non_snake_case)]
#![allow(dead_code)]

use std::process;
use std::sync::Mutex;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::thread::{self, JoinHandle };

use lua::lua_State;
use luakit::{ BaseCodec, Codec, CodecError, LuaBuf, Luakit, LuaPushFn, LuaPushLuaFn, steady_ms };

pub trait IScheduler {
    fn broadcast(&mut self, L: *mut lua_State) -> i32;
    fn call(&mut self, L: *mut lua_State, name: &str) -> i32;
}

pub struct WorkerCodec {
    base: BaseCodec,
    mutex: Mutex<()>,
    write: LuaBuf,
    read: LuaBuf,
}

impl WorkerCodec {
    pub fn new() -> Self {
        Self {
            read: LuaBuf::new(),
            write: LuaBuf::new(),
            mutex: Mutex::new(()),
            base: BaseCodec::new()
         }
    }

    pub fn update(&mut self) ->bool {
        if self.read.empty() {
            if self.write.empty() {
                return false;
            }
            let _g = self.mutex.lock().unwrap();
            std::mem::swap(&mut self.write, &mut self.read);
        }
        true
    }

    pub fn call(&mut self, data: &[u8]) ->bool {
        if let Some(_) = self.write.peek_space(data.len() + 4) {
            self.write.write::<u32>(&(data.len() as u32));
            self.write.push_data(data);
            return true;
        }
        return false;
    }

    pub fn cleanup(&mut self, len: i32) {
        if len > 0 {
            self.read.pop_size(len as usize);
            return;
        }
        self.read.clean();
    }

    pub fn touch(&mut self) -> Option<i32> {
        let slice = self.read.get_slice(None, None);
        let len = self.load_packet(&slice);
        if len > 0 {
            return Some(len);
        }
        None
    }

}

impl Deref for WorkerCodec {
    type Target = BaseCodec;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for WorkerCodec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl Codec for WorkerCodec {
    fn decode(&mut self, L: *mut lua_State) -> Result<i32, CodecError>{
        let mut slice = self.read.get_slice(None, None);
        self.base.decode_impl(L, &mut slice)
    }
}

type Environs = HashMap<String, String>;

pub struct Worker {
    m_stop: bool,
    m_running: bool,
    m_lua: Luakit,
    m_name: String,
    m_mutex: Mutex<()>,
    m_namespace: String,
    m_codec: WorkerCodec,
    m_environs: Environs,
    m_thread: Option<JoinHandle<()>>,
    m_scheduler: *mut dyn IScheduler,
}

#[derive(Debug, Clone)]
pub struct WorkerWrapper(*mut Worker);
impl WorkerWrapper {
    /// 安全条件：调用时指针必须有效且独占
    pub unsafe fn as_mut(&mut self) -> &mut Worker {
        &mut *self.0
    }
    pub fn new(worker: Box<Worker>) -> Self {
        let ptr = Box::into_raw(worker);
        WorkerWrapper(ptr)
    }
    pub fn unwrap(self) -> Box<Worker> {
        let ptr = self.0;
        unsafe { Box::from_raw(ptr) }
    }
}
unsafe impl Send for WorkerWrapper {}
unsafe impl Sync for WorkerWrapper {}

impl Drop for Worker {
    fn drop(&mut self) {
        self.m_lua.close();
    }
}

impl Worker {
    pub fn new(scheduler: *mut dyn IScheduler, name: String, namespace: String) -> Self {
        Self {
            m_name : name,
            m_stop : false,
            m_running : true,
            m_thread : None,
            m_lua : Luakit::new(),
            m_scheduler: scheduler,
            m_namespace : namespace,
            m_mutex: Mutex::new(()),
            m_environs : HashMap::new(),
            m_codec : WorkerCodec::new(),
        }
    }

    pub fn call(&mut self, data: &[u8]) ->bool {
        self.m_codec.call(data)
    }

    pub fn running(&self) -> bool {
        self.m_running
    }

    pub fn stop(&mut self) {
        self.m_stop = true;
        if self.m_thread.is_some() {
            self.m_thread.take().unwrap().join().unwrap();
        }
    }

    pub fn set_env(&mut self, key: &str, val: &str, over : bool) {
        if over || !self.m_environs.contains_key(key) {
            self.m_environs.insert(key.to_string(), val.to_string());
        }
    }

    pub fn get_env(&mut self, key: &str) -> Option<String> {
        match self.m_environs.get(key) {
            Some(val) => Some(val.clone()),
            None => None,
        }
    }

    pub fn add_path(&mut self, field : &str, path: &str) {
        match self.m_environs.get(field) {
            Some(val) => {
                let mut new_path = val.clone();
                new_path.push_str(path);
                self.m_lua.set_path(field, new_path.as_str());
                self.m_environs.insert(field.to_string(), new_path);
            }
            None => {
                self.m_lua.set_path(field, path);
                self.m_environs.insert(field.to_string(), path.to_string());
            }
        }
    }

    pub fn set_path(&mut self, field : &str, path : &str) {
        self.m_lua.set_path(field, path);
    }

    pub fn update(&mut self, clock_ms: u64) {
        if !self.m_codec.update() {
            return;
        }
        while let Some(packet_len) = self.m_codec.touch() {
            // m_lua->table_call(ns, "on_worker", nullptr, &m_codec, std::tie());
            // if (m_codec.failed()) {
            //     m_read_buf->clean();
            //     break;
            // }
            let x = self.m_codec.decode(self.m_lua.L());
            if x.is_err() {
                self.m_codec.cleanup(0);
                break;
            }
            self.m_codec.cleanup(packet_len);
            if steady_ms() - clock_ms > 100 {
                break;
            }
        }
    }

    pub fn setup(&mut self, old_envs: Environs, new_envs: Environs, conf: String) {
        if !conf.is_empty() {
            for (key, value) in &new_envs {
                let ekey = format!("QUANTA_{}", key).to_uppercase();
                self.set_env(&ekey, &value, true);
            }
            let mut global = self.m_lua.get::<luakit::LuaTable>("_G").unwrap();
            self.m_lua.set("platform", luakit::get_platform());
            luakit::set_function!(global, "set_env", |key : String, val : String | {
                return self.set_env(key.as_str(), val.as_str(), true);
            });
            luakit::set_function!(global, "add_path", |field : String, val : String | {
                return self.add_path(field.as_str(), val.as_str());
            });
            luakit::set_function!(global, "set_path", |field : String, val : String | {
                return self.set_path(field.as_str(), val.as_str());
            });
            match self.m_lua.run_script(format!("dofile('{}')\0", conf)) {
                Ok(_) => {},
                Err(e) => println!("load conf Error: {}", e),
            }
        } else {
            if let Some(path) = old_envs.get("LUA_PATH") {
                self.m_lua.set_path("LUA_PATH", path);
            }
            self.m_environs = old_envs;
            for (key, value) in &new_envs {
                let ekey = format!("QUANTA_{}", key).to_uppercase();
                self.set_env(&ekey, &value, true);
            }
        }
    }
    
    pub fn init(&mut self) -> bool {
        let mut quanta = self.m_lua.new_table(Some(&self.m_namespace));
        quanta.set("master", true);
        quanta.set("thread", "quanta");
        quanta.set("pid", process::id());
        quanta.set("tid", thread::current().id());
        quanta.set("platform", luakit::get_platform());
        quanta.set("environs", self.m_environs.clone());
        luakit::set_function!(quanta, "stop", || self.m_running = false );
        luakit::set_function!(quanta, "update", |clock_ms| self.update(clock_ms));
        luakit::set_function!(quanta, "getenv", |key : String | self.get_env(key.as_str()));
        luakit::set_function!(quanta, "setenv", |key : String, val : String | {
            self.set_env(key.as_str(), val.as_str(), true);
        });
        luakit::set_function!(quanta, "call", |L: *mut lua_State, name: String| {
            return unsafe { (*self.m_scheduler).call(L, &name) };
        });
        luakit::set_function!(quanta, "broadcast", |L: *mut lua_State| {
            return unsafe { (*self.m_scheduler).broadcast(L) };
        });
        if let Some(sandbox) = self.get_env("QUANTA_SANDBOX") {
            if let Some(entry) = self.get_env("QUANTA_ENTRY") {
                if let Err(e) = self.m_lua.run_script(format!("require '{}'\0", sandbox)) {
                    println!("require sandbox Error: {}", e);
                    return false;
                }
                if let Err(e) = self.m_lua.run_script(format!("require '{}'\0", entry)) {
                    println!("require entry {} error: {}", entry, e);
                    return false;
                }
                return true;
            }
        }
        true
    }

    pub fn run(&mut self) -> bool {
        if self.m_stop {
            let _ = self.m_lua.table_call(&self.m_namespace, "stop");
            self.m_running = false;
        }
        let _ = self.m_lua.table_call(&self.m_namespace, "run");
        self.m_running
    }

    pub fn start(&mut self, mut wrappr: WorkerWrapper) {
        if self.m_thread.is_some() {
            return;
        }
        self.m_thread = Some(thread::spawn(move || {
            let worker = unsafe { wrappr.as_mut() };
            worker.run();
        }));
    }

}