#![allow(non_snake_case)]
#![allow(dead_code)]

use dashmap::DashMap;
use std::sync::Mutex;
use std::collections::HashMap;

use lua::lua_State;
use libc::c_int as int;
use luakit::{ Codec, LuaRead, LuaTable, Luakit, PtrWrapper };
use crate::worker::{ lua_table_call, IScheduler, Worker, WorkerCodec};

type Environs = HashMap<String, String>;

pub struct Scheduler {
    m_last_tick: u64,
    m_mutex: Mutex<()>,
    m_namespace: String,
    m_codec: WorkerCodec,
    m_environs: Environs,
    m_workers: DashMap<String, PtrWrapper<Worker>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler{
            m_last_tick: 0,
            m_mutex: Mutex::new(()),
            m_workers : DashMap::new(),
            m_environs : HashMap::new(),
            m_codec : WorkerCodec::new(),
            m_namespace : "".to_string(),
        }
    }

    pub fn setup(&mut self, L: *mut lua_State, namespace: String) -> bool {
        let kit = Luakit::load(L);
        if let Some(quanta) = kit.get::<LuaTable>(&namespace) {
            if let Some(envs) = quanta.get::<&str, Environs>("environs"){
                self.m_namespace = namespace;
                self.m_environs = envs;
                return true;
            }
        }
        false
    }

    pub fn startup(&mut self, L: *mut lua_State, name: String, conf: String) -> bool {
        if self.m_workers.contains_key(&name) {
            return false;
        }
        let envs: Environs = LuaRead::lua_to_native(L, 3).unwrap();
        let mut workor = Box::new(Worker::new(self, name.clone(), self.m_namespace.clone()));
        workor.setup(self.m_environs.clone(), envs, conf);
        let wapper = PtrWrapper::new(workor);
        self.m_workers.insert(name, wapper.clone());
        let mut worker = wapper.clone();
        worker.start(wapper.clone());
        true
    }
    
    pub fn stop(&mut self, name: String) {
        if let Some((_, wapper)) = self.m_workers.remove(&name) {
            let mut worker = wapper.clone().unwrap();
            worker.stop();
        }
    }

    
    pub fn check_worker(&mut self) {
        self.m_workers.retain(|_, wapper| {
            let mut worker = wapper.clone();
            if !worker.running() {
                worker.stop();
                return false;
            }
            true
        });
    }

    pub fn shutdown(&mut self) {
        self.m_workers.retain(|_, wapper| {
            let mut worker = wapper.clone().unwrap();
            worker.stop();
            false
        });
    }
    pub fn update(&mut self, L: *mut lua_State, clock_ms: u64) {
        if clock_ms - self.m_last_tick > 1000 {
            self.m_last_tick = clock_ms;
            self.check_worker();
        }
        if !self.m_codec.update() {
            return;
        }
        while let Some(packet_len) = self.m_codec.touch() {
            if !lua_table_call(L, &mut self.m_codec, &self.m_namespace, "on_scheduler") {
                self.m_codec.cleanup(0);
                break;
            }
            self.m_codec.cleanup(packet_len);
            // rust 的steady_ms 实现有BUG，暂时屏蔽
            // if steady_ms() - clock_ms > 100 {
            //     break;
            // }
        }
    }
}

impl IScheduler for Scheduler {
    fn broadcast(&mut self, L: *mut lua_State) -> int{
        let data = self.m_codec.encode(L, 2);
        for mut worker in self.m_workers.iter_mut() {
            worker.call(&data);
        }
        0
    }
    fn call(&mut self, L: *mut lua_State, name: &str) -> int{
        let data = self.m_codec.encode(L, 2);
        if name == "master" {
            let ok = self.m_codec.call(&data);
            unsafe{ lua::lua_pushboolean(L, ok as i32) };
            return 1;
        }
        if let Some(mut worker) = self.m_workers.get_mut(name) {
            let ok = worker.call(&data);
            unsafe{ lua::lua_pushboolean(L, ok as i32) };
            return 1;
        }
        unsafe{ lua::lua_pushboolean(L, 0) };
        1
    }
}