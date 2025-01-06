#![allow(non_snake_case)]
#![allow(dead_code)]

use libc::c_int as int;
use libc::c_char as char;
use lua::{ lua_State, lua_CFunction };
use std::marker::PhantomData;

use crate::lua_function::*;
use crate::lua_base::LuaGuard;
use crate::{ lua_wrapper_function_impl, lua_call_function_impl };
use crate::lua_reference::Reference;
use crate::lua_stack::{LuaRead, LuaPush};

pub struct LuaTable {
    m_index: int,
    m_L: *mut lua_State,
}

impl LuaTable {
    pub fn new(L: *mut lua_State) -> LuaTable {
        unsafe { LuaTable {  m_L: L, m_index: lua::luaL_ref(L, lua::LUA_REGISTRYINDEX) } }
    }
    
    pub fn newref(L: *mut lua_State, index: int) -> LuaTable {
        unsafe {
            lua::lua_pushvalue(L, index);
            LuaTable {  m_L: L, m_index: lua::luaL_ref(L, lua::LUA_REGISTRYINDEX) }
        }
    }

    pub fn get<K, R>(&mut self, key: K) -> Option<R> where K: LuaPush, R: LuaRead {
        let _ = LuaGuard::new(self.m_L);
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

    pub fn get_function(&mut self, function: *const char) -> bool {
        unsafe { 
            lua::lua_rawgeti(self.m_L, lua::LUA_REGISTRYINDEX, self.m_index); 
            lua::lua_getfield(self.m_L, -1, function);
            lua::lua_remove(self.m_L, -2);
            return lua::lua_isfunction(self.m_L, -1);
        }
    }

    pub fn set_function(&mut self, name: *const char, f : lua_CFunction) {
        let _ = LuaGuard::new(self.m_L);
        unsafe { 
            lua::lua_rawgeti(self.m_L, lua::LUA_REGISTRYINDEX, self.m_index); 
            lua::lua_pushcfunction(self.m_L, f);
            lua::lua_setfield(self.m_L, -2, name);
        }
    }

    pub fn call_function(&mut self) ->Result<bool, String> {
        return lua_call_function(self.m_L, 0, 0);
    }

    pub fn call_table(&mut self, func: *const char) ->Result<bool, String> {
        if !self.get_function(func) {
            return Err("function not found".to_string());
        }
        return lua_call_function(self.m_L, 0, 0);
    }

    lua_call_function_impl!(call, );
    lua_call_function_impl!(call1, A);
    lua_call_function_impl!(call2, A, B);
    lua_call_function_impl!(call3, A, B, C);
    lua_call_function_impl!(call4, A, B, C, D);
    lua_call_function_impl!(call5, A, B, C, D, E);
    lua_call_function_impl!(call6, A, B, C, D, E, F);
    lua_call_function_impl!(call7, A, B, C, D, E, F, G);
    lua_call_function_impl!(call8, A, B, C, D, E, F, G, H);
    lua_call_function_impl!(call9, A, B, C, D, E, F, G, H, I);
    lua_call_function_impl!(call10, A, B, C, D, E, F, G, H, I, J);

    lua_wrapper_function_impl!(bind_function,);
    lua_wrapper_function_impl!(bind_function1, A);
    lua_wrapper_function_impl!(bind_function2, A, B);
    lua_wrapper_function_impl!(bind_function3, A, B, C);
    lua_wrapper_function_impl!(bind_function4, A, B, C, D);
    lua_wrapper_function_impl!(bind_function5, A, B, C, D, E);
    lua_wrapper_function_impl!(bind_function6, A, B, C, D, E, F);
    lua_wrapper_function_impl!(bind_function7, A, B, C, D, E, F, G);
    lua_wrapper_function_impl!(bind_function8, A, B, C, D, E, F, G, H);
    lua_wrapper_function_impl!(bind_function9, A, B, C, D, E, F, G, H, I);
    lua_wrapper_function_impl!(bind_function10, A, B, C, D, E, F, G, H, I, J);
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
        unsafe { lua::lua_rawgeti(L, lua::LUA_REGISTRYINDEX, self.m_index) }
        1
    }
}

impl LuaRead for LuaTable {
    fn lua_to_native(L: *mut lua_State, index: i32) -> Option<LuaTable> {
        Some(LuaTable::newref(L, index))
    }
}
