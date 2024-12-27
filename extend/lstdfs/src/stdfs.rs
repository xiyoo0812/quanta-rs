#![allow(non_snake_case)]
#![allow(dead_code)]

use libc::c_int as int;

use lua::{ cstr, lua_State };

pub unsafe fn decode_impl(L: *mut lua_State) -> int {
    return 0;
}

pub unsafe fn encode_impl(L: *mut lua_State) -> int {
    return 0;
}