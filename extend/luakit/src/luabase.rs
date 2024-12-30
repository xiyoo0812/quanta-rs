#![allow(dead_code)]

use lua::lua_State;
use libc::c_int as int;

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

unsafe fn is_lua_array(L: *mut lua_State, idx: i32, emy_as_arr: bool) -> bool {
    if lua::lua_type(L, idx) != lua::LUA_TTABLE { return false; }
    let raw_len = lua::lua_rawlen(L, idx) as isize;
    if raw_len == 0 && !emy_as_arr { return false; }
    let index = lua::lua_absindex(L, idx);
    lua::lua_pushnil(L);
    let mut curlen : isize = 0;
    while lua::lua_next(L, index) != 0 {
        if lua::lua_isinteger(L, -2) == 0 { return false; }
        let key = lua::lua_tointeger(L, -2);
        if key <= 0 || key > raw_len { return false; }
        lua::lua_pop(L, 1);
        curlen += 1;
    }
    return curlen == raw_len;
}
