#![allow(non_snake_case)]
#![allow(dead_code)]

use lua::cstr;
use luakit::Luakit;

pub struct Quanta {
    m_lua: Luakit,
}

impl Drop for Quanta {
    fn drop(&mut self) {
        self.m_lua.close();
    }
}

impl Quanta {
    pub fn new() -> Quanta {
        Quanta {
            m_lua: Luakit::new(),
        }
    }

    pub fn luakit(&mut self) -> &mut Luakit {
        &mut self.m_lua
    }

    pub fn init(&mut self) ->bool {
        let entry = self.m_lua.run_file(cstr!("entry.lua"));
        if entry.is_err() {
            println!("run file Error: {}", entry.unwrap());
            return false;
        }
        self.m_lua.set_function(cstr!("set_path"), |L| {
            let mut kit = Luakit::load(L);
            let field = lua::lua_tolstring(L, 1).unwrap();
            let path = lua::lua_tolstring(L, 2).unwrap();
            kit.set_path(field.as_str(), path.as_str());
            return 0;
        });
        let startup = self.m_lua.call_global(cstr!("startup"));
        if startup.is_err() {
            println!("call startup Error: {}", startup.unwrap());
            return false;
        }
        return true;
    }

    pub fn set_path(&mut self, field : &str, path : &str) {
        self.m_lua.set_path(field, path);
    }

    pub fn run(&mut self) {
        while self.m_lua.get_function(cstr!("run")) {
            let res = self.m_lua.call_function();
            if res.is_err() {
                println!("Error: {}", res.unwrap());
                break;
            }
        }
    }

    pub fn test(&mut self) {
        self.m_lua.set(cstr!("wrapper"), luakit::wrapper_fn(||{ 
            println!("wrapper_fn"); return 1001; 
        }));
        self.m_lua.set(cstr!("wrapper1"), luakit::wrapper_fn1(|a : i32|{ 
            println!("wrapper_fn1 {} {}", a, a * 2); return "abcdefg";
        }));
        self.m_lua.set(cstr!("wrapper2"), luakit::wrapper_fn2(|a : i32, b : i32|{ 
            println!("wrapper_fn2 {} {} {}", a, b, a+b); return 1002;
        }));
        
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
