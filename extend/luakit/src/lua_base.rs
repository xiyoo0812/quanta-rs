#![allow(non_snake_case)]
#![allow(dead_code)]

use lua::lua_State;
use libc::c_int as int;
use std::{env, ops::{Deref, DerefMut}, ptr};

pub struct LuaGuard {
    m_top: int,
    m_L: *mut lua_State,
}

impl LuaGuard {
    pub fn new(L: *mut lua_State) -> LuaGuard {
        let top = unsafe { lua::lua_gettop(L) };
        LuaGuard { m_top: top, m_L: L }
    }
}

impl Drop for LuaGuard {
    fn drop(&mut self) {
        unsafe { lua::lua_settop(self.m_L, self.m_top); }
    }
}

pub fn get_platform() -> &'static str {
    match env::consts::OS {
        "macos" => "apple",
        "windows" => "windows",
        "linux" => "linux",
        _ => "unknown",
    }
}

pub fn lua_get_meta_name<T>() -> String {
    format!("__lua_class_meta_{}__", std::any::type_name::<T>())
}

pub fn is_lua_array(L: *mut lua_State, idx: i32, emy_as_arr: bool) -> bool {
    if !lua::lua_istable(L, idx) {
        return false;
    }
    let raw_len = unsafe { lua::lua_rawlen(L, idx) as isize };
    if raw_len == 0 && !emy_as_arr {
        return false;
    }
    let _gl = LuaGuard::new(L);
    let index = unsafe { lua::lua_absindex(L, idx) };
    unsafe { lua::lua_pushnil(L) };
    let mut curlen : isize = 0;
    while unsafe { lua::lua_next(L, index) } != 0 {
        if lua::lua_isinteger(L, -2) {
            return false;
        }
        let key = lua::lua_tointeger(L, -2);
        if key <= 0 || key > raw_len {
            return false;
        }
        lua::lua_pop(L, 1);
        curlen += 1;
    }
    return curlen == raw_len;
}

pub struct PtrBox<T>{
    pub ptr : *mut T
}

impl<T> PtrBox<T> {
    pub fn new(obj: T) -> Self {
        PtrBox { ptr: Box::into_raw(Box::new(obj)) }
    }
    pub fn load(ptr: *mut T) -> Self {
        PtrBox { ptr: ptr }
    }
    pub fn null() -> Self {
        PtrBox { ptr: ptr::null_mut() }
    }
    pub fn is_null(&self) -> bool {
        self.ptr.is_null()
    }
    pub fn unwrap(self) -> Box<T> {
        unsafe { Box::from_raw(self.ptr) }
    }
}
unsafe impl<T> Send for PtrBox<T> {}
unsafe impl<T> Sync for PtrBox<T> {}

impl<T> Deref for PtrBox<T>  {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe{ &(*self.ptr) }
    }
}

impl<T> Clone for PtrBox<T> {
    fn clone(&self) -> Self {
        PtrBox { ptr: self.ptr }
    }
}

impl<T> DerefMut for PtrBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe{ &mut *self.ptr }
    }
}
