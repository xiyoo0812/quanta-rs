#![allow(non_snake_case)]
#![allow(dead_code)]

use std::sync::Mutex;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::thread::{ JoinHandle };

use lua::lua_State;
use luakit::{ BaseCodec, Codec, CodecError, LuaBuf, Luakit, steady_ms };

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


struct Worker {
    m_stop: bool,
    m_running: bool,
    m_lua: Luakit,
    m_name: String,
    m_mutex: Mutex<()>,
    m_platform: String,
    m_namespace: String,
    m_codec: WorkerCodec,
    m_thread: Option<JoinHandle<()>>,
    m_environs: HashMap<String, String>,
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.m_lua.close();
    }
}

impl Worker {
    pub fn new() -> Self {
        Self {
            m_stop : false,
            m_running : true,
            m_thread : None,
            m_lua : Luakit::new(),
            m_mutex: Mutex::new(()),
            m_name : "".to_string(),
            m_platform : "".to_string(),
            m_namespace : "".to_string(),
            m_codec : WorkerCodec::new(),
            m_environs : HashMap::new(),
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

    pub fn update(&mut self, clock_ms: u64) {
        if !self.m_codec.update() {
            return;
        }
        while let Some(packet_len) = self.m_codec.touch() {
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

    pub fn run(&mut self) {
        self.m_running = true;
        while !self.m_stop {}
    }

}