#![allow(non_snake_case)]
#![allow(dead_code)]

use libc::c_int as int;

use lua::{ cstr, lua_State };

pub struct luakit {
    m_L: *mut lua_State,
}

