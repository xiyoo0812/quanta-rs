#![allow(non_snake_case)]
#![allow(dead_code)]

use std::thread;
use std::process;
use std::sync::Arc;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

use lua::cstr;
use signal_hook::flag;
use libc::c_char as char;

use luakit::Luakit;

pub struct Quanta {
    m_lua: Luakit,
    m_signal: i32,
    m_environs: HashMap<String, String>,
    m_signals: HashMap<i32, Arc<AtomicBool>>,
}

impl Drop for Quanta {
    fn drop(&mut self) {
        self.m_lua.close();
    }
}

impl Quanta {
    pub fn new() -> Quanta {
        Quanta {
            m_signal : 0,
            m_lua: Luakit::new(),
            m_environs: HashMap::new(),
            m_signals: HashMap::new(),
        }
    }

    pub fn luakit(&mut self) -> &mut Luakit {
        &mut self.m_lua
    }

    pub fn set_signal(&mut self, n: i32, b: bool) {
        let mask = 1 << n;
        if b {
            self.m_signal |= mask;
        } else {
            self.m_signal ^= mask;
        }
    }
    
    pub fn set_env(&mut self, key: &str, val: &str, over : bool) {
        if over && !self.m_environs.contains_key(key) {
            self.m_environs.insert(key.to_string(), val.to_string());
        }
    }

    pub fn get_env(&mut self, key: &str) -> String {
        match self.m_environs.get(key) {
            Some(val) => val.clone(),
            None => String::new(),
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

    pub fn setup(&mut self, argv: Vec<String>) ->bool {
        //声明函数
        let mut global = self.m_lua.new_table(Some(cstr!("_G")));
        global.set(cstr!("platform"), luakit::get_platform());
        global.bind_function2(cstr!("set_env"), |key : String, val : String | { 
            return self.set_env(key.as_str(), val.as_str(), true);
        });
        global.bind_function2(cstr!("add_path"), |field : String, val : String | { 
            return self.add_path(field.as_str(), val.as_str());
        });
        global.bind_function2(cstr!("set_path"), |field : String, val : String | { 
            return self.set_path(field.as_str(), val.as_str());
        });
        //加载参数
        for (i, arg) in argv.iter().enumerate().skip(1) {
            if let Some((key, value)) = arg.split_once('=') {
                if key.starts_with("--") {
                    let new_key = format!("QUANTA_{}", &key[2..]).to_uppercase();
                    self.set_env(new_key.as_str(), value, true);
                    continue;
                }
            }
            if i == 1 {
                match self.m_lua.run_file(arg.as_ptr() as *const char) {
                    Ok(_) => {},
                    Err(e) => {
                        println!("load conf Error: {}", e);
                        return false;
                    }
                }
            }
        }
        return true;
    }

    pub fn init(&mut self) ->bool {
        //设置环境以及初始一些函数
        let mut quanta = self.m_lua.new_table(Some(cstr!("quanta")));
        quanta.set("master", true);
        quanta.set("thread", "quanta");
        quanta.set("pid", process::id());
        quanta.set("tid", thread::current().id());
        quanta.set("platform", luakit::get_platform());
        quanta.set("environs", self.m_environs.clone());
        quanta.bind_function1(cstr!("getenv"), |key : String | {
            return self.get_env(key.as_str());
        });
        quanta.bind_function2(cstr!("setenv"), |key : String, val : String | {
            self.set_env(key.as_str(), val.as_str(), true);
        });
        quanta.bind_function1(cstr!("ignore_signal"), |_ : i32 | {});
        quanta.bind_function1(cstr!("default_signal"), |_ : i32 | {});
        quanta.bind_function1(cstr!("register_signal"), | n : i32 | {
            let signal = Arc::new(AtomicBool::new(false));
            match flag::register(n, Arc::clone(&signal)) {
                Ok(_) => {
                    self.m_signals.insert(n, Arc::clone(&signal));
                },
                Err(e) => {
                    println!("register signal Error: {}", e);
                }
            };
        });
        quanta.bind_function(cstr!("get_singal"), || {
            let mut signal_mask = self.m_signal;
            for (n, signal) in self.m_signals.iter() {
                if signal.load(Ordering::Relaxed) {
                    signal_mask |= 1 << n;
                }
            }
            return signal_mask;
        });
        
        //设置日志
        let env_log_path = self.get_env("QUANTA_LOG_PATH");
        if !env_log_path.is_empty() {
            let _env_index = self.get_env("QUANTA_INDEX");
            let _env_service = self.get_env("QUANTA_SERVICE");
            //logger::get_logger()->option(env_log_path, env_service, env_index);
        }
        //加载sandbox和entry
        let sandbox = self.get_env("QUANTA_SANDBOX");
        let entry = self.get_env("QUANTA_ENTRY");
        if sandbox.is_empty() || entry.is_empty() {
            println!("the sandbox or entry not found");
            return false;
        }
        match self.m_lua.run_file(sandbox.as_ptr() as *const char) {
            Ok(_) => {},
            Err(e) => {
                println!("run sandbox Error: {}", e);
                return false;
            }
        }
        match self.m_lua.run_file(entry.as_ptr() as *const char) {
            Ok(_) => {},
            Err(e) => {
                println!("run entry Error: {}", e);
                return false;
            }
        }
        return true;
    }

    pub fn run(&mut self) {
        if self.init() {
            self.test();
            //主循环
            while self.m_lua.get_function(cstr!("run")) {
                match self.m_lua.call_function(){
                    Ok(_) => {},
                    Err(e) => {
                        println!("Error: {}", e);
                        break;
                    }
                }
            }
        }
    }

    pub fn test(&mut self) {
        self.m_lua.bind_function(cstr!("wrapper"), ||{ 
            println!("func_wrapper"); return 1001; 
        });
        self.m_lua.bind_function1(cstr!("wrapper1"), |a : i32|{ 
            println!("func_wrapper1 {} {}", a, a * 2); return "abcdefg";
        });
        self.m_lua.bind_function2(cstr!("wrapper2"), |a : i32, b : i32|{ 
            println!("func_wrapper2 {} {} {}", a, b, a+b); return 1002;
        });
        
        let res = self.m_lua.call_global(cstr!("test"));
        match res {
            Ok(_) => println!("call_global executed successfully"),
            Err(e) => println!("Error: {}", e),
        }
        let ret = self.m_lua.call2(cstr!("test1"), 3, 1, 2);
        match ret {
            Ok(mut refs) => {
                let r1 = refs[0].get::<i32>().unwrap();
                let r2 = refs[1].get::<i32>().unwrap();
                let r3 = refs[2].get::<String>().unwrap();
                println!("call_lua1 ret {}: {}, {}", r1, r2, r3);
            }
            Err(e) => println!("Error: {}", e),
        }
    }
}
