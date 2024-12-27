#![allow(non_snake_case)]

extern crate libc;
extern crate lua;

#[macro_use]
mod macros;

mod json;

use lua::lua_State;
use libc::c_int as int;

extern "C" fn encode(L: *mut lua_State) -> int {
    unsafe { return json::encode_impl(L); }
}

extern "C" fn decode(L: *mut lua_State) -> int {
    unsafe { return json::decode_impl(L); }
}

#[no_mangle]
pub extern "C" fn luaopen_ljson(L: *mut lua_State) -> int {
    unsafe{
        lua::lua_createtable(L, 4, 0);
        lua::lua_pushcfunction(L, encode);
        lua::lua_setfield(L, -2, cstr!("encode"));
        lua::lua_pushcfunction(L, decode);
        lua::lua_setfield(L, -2, cstr!("decode"));
        return 1;
    }
}
