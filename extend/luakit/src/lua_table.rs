#![allow(non_snake_case)]
#![allow(dead_code)]

use libc::c_int as int;
use lua::{ to_char, lua_State, lua_CFunction };

use crate::lua_function::*;
use crate::lua_base::LuaGuard;
use crate::lua_reference::Reference;
use crate::lua_stack::{LuaRead, LuaPush, LuaPushFn, LuaPushLuaFn};
use crate::{ lua_set_function, lua_set_lua_function, call_lua_warper, call_function_inner };

pub struct LuaTable {
    m_index: int,
    m_L: *mut lua_State,
}

impl LuaTable {
    pub fn new(L: *mut lua_State) -> LuaTable {
        unsafe { LuaTable {  m_L: L, m_index: lua::luaL_ref(L, lua::LUA_REGISTRYINDEX) } }
    }

    pub fn load(L: *mut lua_State, index: int) -> LuaTable {
        unsafe {
            lua::lua_pushvalue(L, index);
            LuaTable {  m_L: L, m_index: lua::luaL_ref(L, lua::LUA_REGISTRYINDEX) }
        }
    }

    pub fn L(&mut self) -> *mut lua_State {
        return self.m_L;
    }

    pub fn get<K, R>(&mut self, key: K) -> Option<R> where K: LuaPush, R: LuaRead {
        let _gl = LuaGuard::new(self.m_L);
        unsafe { lua::lua_rawgeti(self.m_L, lua::LUA_REGISTRYINDEX, self.m_index); }
        key.native_to_lua(self.m_L);
        unsafe { lua::lua_gettable(self.m_L, -2); }
        return LuaRead::lua_to_native(self.m_L, -1);
    }

    pub fn set<K, V>(&mut self, key: K, val: V) where K: LuaPush, V: LuaPush {
        unsafe { lua::lua_rawgeti(self.m_L, lua::LUA_REGISTRYINDEX, self.m_index); }
        key.native_to_lua(self.m_L);
        val.native_to_lua(self.m_L);
        unsafe { lua::lua_settable(self.m_L, -3); }
    }

    pub fn new_table(&mut self, name: Option<&str>) -> LuaTable {
        let _gl = LuaGuard::new(self.m_L);
        unsafe {
            lua::lua_rawgeti(self.m_L, lua::LUA_REGISTRYINDEX, self.m_index);
            lua::lua_createtable(self.m_L, 0, 8);
            if !name.is_none() {
                lua::lua_pushvalue(self.m_L, -1);
                lua::lua_setfield(self.m_L, -3, to_char!(name.unwrap()));
            }
            LuaTable::new(self.m_L)
        }
    }

    pub fn get_function(&mut self, function: &str) -> bool {
        unsafe {
            lua::lua_rawgeti(self.m_L, lua::LUA_REGISTRYINDEX, self.m_index);
            lua::lua_getfield(self.m_L, -1, to_char!(function));
            lua::lua_remove(self.m_L, -2);
            return lua::lua_isfunction(self.m_L, -1);
        }
    }

    pub fn set_function(&mut self, name: &str, f : lua_CFunction) {
        let _gl = LuaGuard::new(self.m_L);
        unsafe {
            lua::lua_rawgeti(self.m_L, lua::LUA_REGISTRYINDEX, self.m_index);
            lua::lua_pushcfunction(self.m_L, f);
            lua::lua_setfield(self.m_L, -2, to_char!(name));
        }
    }

    pub fn call_function(&mut self) ->Result<bool, String> {
        return lua_call_function(self.m_L, 0, 0);
    }

    pub fn call(&mut self, func: &str) ->Result<bool, String> {
        if !self.get_function(func) {
            return Err("function not found".to_string());
        }
        return lua_call_function(self.m_L, 0, 0);
    }

    call_lua_warper!(call0,);
    call_lua_warper!(call1, A);
    call_lua_warper!(call2, A, B);
    call_lua_warper!(call3, A, B, C);
    call_lua_warper!(call4, A, B, C, D);
    call_lua_warper!(call5, A, B, C, D, E);
    call_lua_warper!(call6, A, B, C, D, E, F);
    call_lua_warper!(call7, A, B, C, D, E, F, G);
    call_lua_warper!(call8, A, B, C, D, E, F, G, H);
    call_lua_warper!(call9, A, B, C, D, E, F, G, H, I);
    call_lua_warper!(call10, A, B, C, D, E, F, G, H, I, J);

    lua_set_function!(set_function0,);
    lua_set_function!(set_function1, A);
    lua_set_function!(set_function2, A, B);
    lua_set_function!(set_function3, A, B, C);
    lua_set_function!(set_function4, A, B, C, D);
    lua_set_function!(set_function5, A, B, C, D, E);
    lua_set_function!(set_function6, A, B, C, D, E, F);
    lua_set_function!(set_function7, A, B, C, D, E, F, G);
    lua_set_function!(set_function8, A, B, C, D, E, F, G, H);
    lua_set_function!(set_function9, A, B, C, D, E, F, G, H, I);
    lua_set_function!(set_function10, A, B, C, D, E, F, G, H, I, J);

    lua_set_lua_function!(set_lfunction0,);
    lua_set_lua_function!(set_lfunction1, A);
    lua_set_lua_function!(set_lfunction2, A, B);
    lua_set_lua_function!(set_lfunction3, A, B, C);
    lua_set_lua_function!(set_lfunction4, A, B, C, D);
    lua_set_lua_function!(set_lfunction5, A, B, C, D, E);
    lua_set_lua_function!(set_lfunction6, A, B, C, D, E, F);
    lua_set_lua_function!(set_lfunction7, A, B, C, D, E, F, G);
    lua_set_lua_function!(set_lfunction8, A, B, C, D, E, F, G, H);
    lua_set_lua_function!(set_lfunction9, A, B, C, D, E, F, G, H, I);
    lua_set_lua_function!(set_lfunction10, A, B, C, D, E, F, G, H, I, J);
}

impl Drop for LuaTable {
    fn drop(&mut self) {
        if self.m_index != lua::LUA_REFNIL && self.m_index != lua::LUA_NOREF {
            unsafe { lua::luaL_unref(self.m_L, lua::LUA_REGISTRYINDEX, self.m_index); }
        }
    }
}

impl LuaPush for LuaTable {
    fn native_to_lua(self, L: *mut lua_State) -> i32 {
        unsafe { lua::lua_rawgeti(L, lua::LUA_REGISTRYINDEX, self.m_index); }
        1
    }
}

impl LuaRead for LuaTable {
    fn lua_to_native(L: *mut lua_State, index: i32) -> Option<LuaTable> {
        Some(LuaTable::load(L, index))
    }
}
