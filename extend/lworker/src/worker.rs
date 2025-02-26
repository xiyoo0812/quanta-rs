#![allow(non_snake_case)]
#![allow(dead_code)]

use std::process;
use std::sync::Mutex;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::thread::{self, JoinHandle };

use lua::lua_State;
use libc::c_int as int;
use luakit::{ BaseCodec, Codec, CodecError, LuaBuf, LuaPushFn, LuaPushLuaFn, Luakit, PtrBox };

type Environs = HashMap<String, String>;

pub trait IScheduler {
    fn broadcast(&mut self, L: *mut lua_State) -> int;
    fn call(&mut self, L: *mut lua_State, name: &str) -> int;
}

pub fn lua_table_call<T: Codec>(L: *mut lua_State, codec: &mut T, table: &str, func: &str) ->bool {
    if luakit::get_table_function(L, table, func) {
        if let Ok(argc) = codec.decode(L) {
            if let Ok(_) = luakit::lua_call_function(L, argc, 0) {
                return true;
            }
        }
    }
    return false;
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
        let mut slice = self.read.get_slice(None, Some(4));
        self.base.decode_impl(L, &mut slice)
    }
}

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

    pub fn update(&mut self, _clock_ms: u64) {
        if !self.m_codec.update() {
            return;
        }
        let L = self.m_lua.L();
        while let Some(packet_len) = self.m_codec.touch() {
            if !lua_table_call(L, &mut self.m_codec, &self.m_namespace, "on_worker") {
                self.m_codec.cleanup(0);
                break;
            }
            self.m_codec.cleanup(packet_len);
            // rust 的steady_ms 实现有BUG，暂时屏蔽
            // if steady_ms() - _clock_ms > 100 {
            //     break;
            // }
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
        quanta.set("pid", process::id());
        quanta.set("tid", thread::current().id());
        quanta.set("thread", self.m_name.clone());
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

    pub fn run(&mut self) {
        while self.m_running {
            if self.m_stop {
                let _ = self.m_lua.table_call(&self.m_namespace, "stop");
                self.m_running = false;
            }
            let _ = self.m_lua.table_call(&self.m_namespace, "run");
        }
    }

    pub fn start(&mut self, mut wrappr: PtrBox<Worker>) {
        if self.m_thread.is_some() {
            return;
        }
        self.m_thread = Some(thread::spawn(move || {
            wrappr.init();
            wrappr.run();
        }));
    }
}
