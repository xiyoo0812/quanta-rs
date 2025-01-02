#![allow(non_snake_case)]
#![allow(dead_code)]

use lua::lua_State;
use libc::c_int as int;

use crate::lua_stack::{LuaRead, LuaPush};

pub struct Reference {
    m_index: int,
    m_L: *mut lua_State,
}

macro_rules! refenerce_get {
    ($name:ident) => {
        pub unsafe fn $name<R>(&mut self) -> Option<R> where R: LuaRead {
            lua::lua_rawgeti(self.m_L, lua::LUA_REGISTRYINDEX, self.m_index);
            let ret = LuaRead::lua_to_native(self.m_L, -1);
            lua::lua_pop(self.m_L, 1);
            return ret;
        }
    }
}

impl Reference {
    pub fn new(L: *mut lua_State, index: int) -> Reference {
        unsafe {
            lua::lua_pushvalue(L, index);
            Reference {  m_L: L, m_index: lua::luaL_ref(L, lua::LUA_REGISTRYINDEX), }
        }
    }

    refenerce_get!(get_i32);
    refenerce_get!(get_u32);
    refenerce_get!(get_i64);
    refenerce_get!(get_u64);
    refenerce_get!(get_f32);
    refenerce_get!(get_f64);
    refenerce_get!(get_str);
    refenerce_get!(get_string);
}

impl Drop for Reference {
    fn drop(&mut self) {
        if self.m_index != lua::LUA_REFNIL && self.m_index != lua::LUA_NOREF {
            unsafe { lua::luaL_unref(self.m_L, lua::LUA_REGISTRYINDEX, self.m_index); }
        }
    }
}

impl LuaPush for Reference {
    fn native_to_lua(self, L: *mut lua_State) -> i32 {
        unsafe { lua::lua_rawgeti(L, lua::LUA_REGISTRYINDEX, self.m_index) }
        1
    }
}

impl LuaRead for Reference {
    fn lua_to_native(L: *mut lua_State, index: i32) -> Option<Reference> {
        Some(Reference::new(L, index))
    }
}
