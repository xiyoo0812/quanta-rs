#![allow(non_snake_case)]
#![allow(dead_code)]

use dashmap::DashMap;
use std::sync::Mutex;
use std::collections::HashMap;

use lua::lua_State;
use luakit::{ Codec, LuaTable, Luakit, steady_ms };
use crate::worker::{ IScheduler, Worker, WorkerCodec, WorkerWrapper};

type Environs = HashMap<String, String>;

pub struct Scheduler {
    m_lua: Luakit,
    m_last_tick: u64,
    m_mutex: Mutex<()>,
    m_namespace: String,
    m_codec: WorkerCodec,
    m_environs: Environs,
    m_workers: DashMap<String, WorkerWrapper>,
}

impl Scheduler {
    pub fn new(L: *mut lua_State, namespace: String) -> Self {
        Scheduler{
            m_last_tick: 0,
            m_namespace : namespace,
            m_lua : Luakit::load(L),
            m_mutex: Mutex::new(()),
            m_environs : HashMap::new(),
            m_codec : WorkerCodec::new(),
            m_workers : DashMap::new(),
        }
    }

    pub fn setup(&mut self) -> bool {
        if let Some(quanta) = self.m_lua.get::<LuaTable>("quanta") {
            if let Some(envs) = quanta.get::<&str, Environs>("environs"){
                self.m_environs = envs;
                return true;
            }
        }
        false
    }

    pub fn startup(&mut self, name: String, envs: Environs, conf: String) -> bool {
        if self.m_workers.contains_key(&name) {
            return false;
        }
        let mut workor = Box::new(Worker::new(self, Luakit::new(), name.clone(), self.m_namespace.clone()));
        workor.setup(self.m_environs.clone(), envs, conf);
        let wapper = WorkerWrapper::new(workor);
        self.m_workers.insert(name, wapper.clone());
        let mut nwapper = wapper.clone();
        let worker = unsafe { nwapper.as_mut() };
        worker.start(wapper);
        true
    }
    
    pub fn stop(&mut self, name: String) {
        if let Some((_, wapper)) = self.m_workers.remove(&name) {
            let mut nwapper = wapper.clone();
            let worker = unsafe { nwapper.as_mut() };
            worker.stop();
        }
    }

    
    pub fn check_worker(&mut self) {
        self.m_workers.retain(|_, wapper| {
            let mut nwapper = wapper.clone();
            let worker = unsafe { nwapper.as_mut() };
            if !worker.running() {
                worker.stop();
                return false;
            }
            true
        });
    }

    pub fn shutdown(&mut self) {
        self.m_workers.retain(|_, wapper| {
            let mut nwapper = wapper.clone();
            let worker = unsafe { nwapper.as_mut() };
            worker.stop();
            false
        });
    }
    pub fn update(&mut self, clock_ms: u64) {
        if clock_ms - self.m_last_tick > 1000 {
            self.m_last_tick = clock_ms;
            self.check_worker();
        }
        while let Some(packet_len) = self.m_codec.touch() {
            // m_lua->table_call(ns, "on_scheduler", nullptr, &m_codec, std::tie());
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
}

impl IScheduler for Scheduler {
    fn broadcast(&mut self, L: *mut lua_State) -> i32{
        let data = self.m_codec.encode(L, 2);
        for wapper in self.m_workers.iter() {
            let mut nwapper = wapper.clone();
            let worker = unsafe { nwapper.as_mut() };
            worker.call(&data);
        }
        0
    }
    fn call(&mut self, L: *mut lua_State, name: &str, data: &[u8]) -> i32{
        if data.len() > 0 {
            if name == "master" {
                let ok = self.m_codec.call(data);
                unsafe{ lua::lua_pushboolean(L, ok as i32) };
                return 1;
            }
            if let Some(wapper) = self.m_workers.get(name) {
                let mut nwapper = wapper.clone();
                let worker = unsafe { nwapper.as_mut() };
                let ok = worker.call(data);
                unsafe{ lua::lua_pushboolean(L, ok as i32) };
                return 1;
            }
        }
        unsafe{ lua::lua_pushboolean(L, 0) };
        1
    }
}