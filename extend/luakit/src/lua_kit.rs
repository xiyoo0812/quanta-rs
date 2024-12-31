#![allow(non_snake_case)]
#![allow(dead_code)]

use lua::lua_State;
use libc::c_char as char;

use crate::lua_function::*;
use crate::lua_base::LuaGuard;

pub struct Luakit {
    m_L: *mut lua_State,
}

impl Luakit {
    pub fn new() -> Luakit {
        unsafe {
            let L =  lua::luaL_newstate();
            lua::luaL_openlibs(L);
            lua::lua_checkstack(L, 1024);
            Luakit { m_L: L }
        }
    }

    pub fn load(L: *mut lua_State) -> Luakit {
        Luakit { m_L: L }
    }

    pub fn L(&mut self) -> *mut lua_State {
        return self.m_L;
    }

    // pub fn set<V>(&mut self, name: &str, value: V) {
    //     native_to_lua(self.m_L, value);
    //     unsafe { lua::lua_setglobal(self.m_L, cstr(name)); }
    // }

    // pub fn get<V>(&mut self, name: &str) -> V {
    //     let _ = LuaGuard::new(self.m_L);
    //     unsafe { lua::lua_getglobal(self.m_L, cstr(name)); }
    //     return lua_to_native<V>(self.m_L, -1);
    // }

    pub unsafe fn run_script(&mut self, script: *const char) ->Result<bool, String> {
        let _ = LuaGuard::new(self.m_L);
        if lua::luaL_loadstring(self.m_L, script) == 1 {
            let err= lua::lua_tostring(self.m_L, -1).unwrap();
            print!("lua loadstring err: {}", err);
            return Err(err);
        }
        return lua_call_function(self.m_L, 0, 0);
    }
}

impl Drop for Luakit {
    fn drop(&mut self) {
        unsafe { lua::lua_close(self.m_L); }
    }
}

