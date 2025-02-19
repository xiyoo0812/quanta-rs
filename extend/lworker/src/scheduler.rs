#![allow(non_snake_case)]
#![allow(dead_code)]

use dashmap::DashMap;
use std::sync::{Arc, Mutex, Weak};
use std::collections::HashMap;

use lua::lua_State;
use luakit::{ Codec, LuaTable, Luakit, steady_ms };
use crate::worker::{ IScheduler, Worker, WorkerCodec};

type Environs = HashMap<String, String>;

struct Scheduler {
    m_lua: Luakit,
    m_last_tick: u64,
    m_mutex: Mutex<()>,
    m_namespace: String,
    m_codec: WorkerCodec,
    m_environs: Environs,
    m_scheduler: Arc<Mutex<dyn IScheduler>>,
    m_workers: DashMap<String, Arc<Mutex<Worker>>>,
}

struct SchedulerImpl {
    scheduler: Weak<Mutex<Scheduler>>,
}

impl Scheduler {
    pub fn new(L: *mut lua_State, namespace: String) -> Arc<Mutex<Self>> {
        Arc::new_cyclic(|weak_self| {
            let scheduler = Arc::new(Mutex::new(SchedulerImpl {
                scheduler: weak_self.clone(),
            })) as Arc<Mutex<dyn IScheduler>>;
            Mutex::new(Scheduler{
                m_last_tick: 0,
                m_namespace : namespace,
                m_lua : Luakit::load(L),
                m_mutex: Mutex::new(()),
                m_environs : HashMap::new(),
                m_codec : WorkerCodec::new(),
                m_workers : DashMap::new(),
                m_scheduler : scheduler,
            })
        })
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
        let clone = self.m_scheduler.clone();
        let mut workor = Worker::new(clone, Luakit::new(), name.clone(), self.m_namespace.clone());
        workor.setup(self.m_environs.clone(), envs, conf);
        let rcworker: Arc<Mutex<Worker>> =  Arc::new(Mutex::new(workor));
        let mut worker = rcworker.lock().unwrap();
        worker.start(rcworker.clone());
        self.m_workers.insert(name, rcworker.clone());
        true
    }
    
    pub fn stop(&mut self, name: String) {
        if let Some((_, worker)) = self.m_workers.remove(&name) {
            let mut worker = worker.lock().unwrap();
            worker.stop();
        }
    }

    
    pub fn check_worker(&mut self) {
        self.m_workers.retain(|_, worker| {
            let mut worker = worker.lock().unwrap();
            if !worker.running() {
                worker.stop();
                return false;
            }
            true
        });
    }

    pub fn shutdown(&mut self) {
        self.m_workers.retain(|_, worker| {
            let mut worker = worker.lock().unwrap();
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

    fn broadcast(&mut self, L: *mut lua_State) -> i32{
        let data = self.m_codec.encode(L, 2);
        for worker in self.m_workers.iter() {
            let mut worker = worker.lock().unwrap();
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
            if let Some(worker) = self.m_workers.get(name) {
                let mut worker = worker.lock().unwrap();
                let ok = worker.call(data);
                unsafe{ lua::lua_pushboolean(L, ok as i32) };
                return 1;
            }
        }
        unsafe{ lua::lua_pushboolean(L, 0) };
        1
    }
}

impl IScheduler for SchedulerImpl {
    fn broadcast(&mut self, L: *mut lua_State) -> i32{
        if let Some(scheduler) = self.scheduler.upgrade() {
            let mut scheduler = scheduler.lock().unwrap();
            scheduler.broadcast(L)
        } else {
            0
        }
    }
    fn call(&mut self, L: *mut lua_State, name: &str, data: &[u8]) -> i32{
        if let Some(scheduler) = self.scheduler.upgrade() {
            let mut scheduler = scheduler.lock().unwrap();
            scheduler.call(L, name, data)
        } else {
            0
        }
    }
}