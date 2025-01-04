#![allow(non_snake_case)]

extern crate lua;
extern crate libc;
extern crate luakit;

mod json;

use libc::c_int as int;
use json::{ encode_impl, decode_impl };
use lua::{ cstr, lua_reg, lua_State };

#[no_mangle]
pub extern "C" fn luaopen_ljson(L: *mut lua_State) -> int {
    let LFUNS = [
        lua_reg!("encode", encode_impl),
        lua_reg!("decode", decode_impl),
        lua_reg!(),
    ];
    unsafe {
        lua::lua_createtable(L, LFUNS.len() as i32, 0);
        lua::luaL_setfuncs(L, LFUNS.as_ptr(), 0);
        return 1;
    }
}
