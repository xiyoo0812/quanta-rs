#![allow(non_snake_case)]
#![allow(dead_code)]

use libc::c_int as int;
use lua::{ cstr, lua_State };
use luabase::LuaGuard as LuaGuard;

pub struct Luakit {
    m_L: *mut lua_State,
}

impl Luakit {
    pub fn new() -> Luakit {
        let mut L = unsafe { lua::luaL_newstate() };
        lua::luaL_openlibs(L);
        lua::lua_checkstack(m_L, 1024);
        Luakit { m_L: L }
    }

    pub fn load(L: *mut lua_State) -> Luakit {
        Luakit { m_L: L }
    }

    pub fn L() -> *mut lua_State {
        return m_L;
    }

    pub fn set<V>(&mut self, name: &str, value: V) {
        native_to_lua(self.m_L, value);
        unsafe { lua::lua_setglobal(self.m_L, cstr(name)); }
    }

    pub fn get<V>(&mut self, name: &str) -> V {
        let _ = LuaGuard::new(self.m_L);
        unsafe { lua::lua_getglobal(self.m_L, cstr(name)); }
        return lua_to_native<V>(self.m_L, -1);
    }
}

impl Drop for Luakit {
    fn drop(&mut self) {
        unsafe { lua::lua_close(self.m_L); }
    }
}

