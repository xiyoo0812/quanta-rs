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
    let l = [
        lua_reg!("encode", |l| { return encode_impl(l);}),
        lua_reg!("decode", |l| { return decode_impl(l);}),
        lua_reg!(),
    ];
    unsafe {
        lua::lua_createtable(L, 2, 0);
        lua::luaL_setfuncs(L, l.as_ptr(), 0);
        return 1;
    }
}
