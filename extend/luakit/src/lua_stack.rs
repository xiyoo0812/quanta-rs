#![allow(non_snake_case)]
#![allow(dead_code)]

use std::rc::Rc;
use std::any::Any;
use std::cell::RefCell;

use libc::c_int as int;
use lua::{ cstr, lua_State };

pub trait LuaNative  {
    unsafe fn push(&self, L: *mut lua_State);
    unsafe fn pull(&mut self, L: *mut lua_State, idx: int);
}

impl LuaNative for bool {
    unsafe fn push(&self, L: *mut lua_State) { lua::lua_pushboolean(L, *self as int); }
    unsafe fn pull(&mut self, L: *mut lua_State, idx: int) { *self = lua::lua_toboolean(L, idx) != 0; }
}
impl LuaNative for i32 {
    unsafe fn push(&self, L: *mut lua_State) { lua::lua_pushinteger(L, *self as lua::lua_Integer); }
    unsafe fn pull(&mut self, L: *mut lua_State, idx: int) { *self = lua::lua_tointeger(L, idx) as i32; }
}
impl LuaNative for u32 {
    unsafe fn push(&self, L: *mut lua_State) { lua::lua_pushinteger(L, *self as lua::lua_Integer); }
    unsafe fn pull(&mut self, L: *mut lua_State, idx: int) { *self = lua::lua_tointeger(L, idx) as u32; }
}
impl LuaNative for i64 {
    unsafe fn push(&self, L: *mut lua_State) { lua::lua_pushinteger(L, *self as lua::lua_Integer); }
    unsafe fn pull(&mut self, L: *mut lua_State, idx: int) { *self = lua::lua_tointeger(L, idx) as i64; }
}
impl LuaNative for u64 {
    unsafe fn push(&self, L: *mut lua_State) { lua::lua_pushinteger(L, *self as lua::lua_Integer); }
    unsafe fn pull(&mut self, L: *mut lua_State, idx: int) { *self = lua::lua_tointeger(L, idx) as u64; }
}
impl LuaNative for f32 {
    unsafe fn push(&self, L: *mut lua_State) { lua::lua_pushnumber(L, *self as lua::lua_Number); }
    unsafe fn pull(&mut self, L: *mut lua_State, idx: int) { *self = lua::lua_tonumber(L, idx) as f32; }
}
impl LuaNative for f64 {
    unsafe fn push(&self, L: *mut lua_State) { lua::lua_pushnumber(L, *self as f64); }
    unsafe fn pull(&mut self, L: *mut lua_State, idx: int) { *self = lua::lua_tonumber(L, idx) as f64; }
}

pub unsafe fn lua_to_native(L: *mut lua_State, idx: int) -> Rc<RefCell<dyn Any>> {
    let ttype = lua::lua_type(L, idx);
    match ttype {
        lua::LUA_TBOOLEAN => Rc::new(RefCell::new(lua::lua_toboolean(L, idx) != 0)),
        lua::LUA_TSTRING => Rc::new(RefCell::new(lua::lua_tolstring(L, idx).unwrap())),
        lua::LUA_TNUMBER => {
            if lua::lua_isinteger(L, idx) == 1 {
                Rc::new(RefCell::new(lua::lua_tointeger(L, idx)))
            } else {
                Rc::new(RefCell::new(lua::lua_tonumber(L, idx)))
            }
        },
        lua::LUA_TTABLE => {
            lua::lua_getfield(L, idx, cstr!("__pointer__"));
            let rc = lua::lua_touserdata(L, -1) as *mut Rc<RefCell<dyn Any>>;
            lua::lua_pop(L, 1);
            return rc.as_ref().unwrap().clone();
        },
        lua::LUA_TLIGHTUSERDATA => {
            let rc = lua::lua_touserdata(L, idx) as *mut Rc<RefCell<dyn Any>>;
            return rc.as_ref().unwrap().clone();
        },
        lua::LUA_TUSERDATA => {
            let rc = lua::lua_touserdata(L, idx) as *mut Rc<RefCell<dyn Any>>;
            return rc.as_ref().unwrap().clone();
        },
        _ => Rc::new(RefCell::new(()))
    }
}

pub unsafe fn lua_to_native_mutil(L: *mut lua_State, len : int) -> Vec<Rc<RefCell<dyn Any>>> {
    let mut vec = Vec::with_capacity(len as usize);
    for i in 0..len {
        vec.push(lua_to_native(L, (i - len) as int));
    }
    return vec;
}
