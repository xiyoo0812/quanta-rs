#![allow(non_snake_case)]

extern crate lua;
extern crate libc;
extern crate luakit;

mod json;

use libc::c_int as int;
use json::{ encode_impl, decode_impl };
use lua::{ cstr, lreg, lreg_null, lua_State };

#[no_mangle]
pub extern "C" fn luaopen_ljson(L: *mut lua_State) -> int {
    let l = [
        lreg!("encode", |l| { return encode_impl(l);}),
        lreg!("decode", |l| { return decode_impl(l);}),
        lreg_null!(),
    ];
    unsafe {
        lua::lua_createtable(L, 2, 0);
        lua::luaL_setfuncs(L, l.as_ptr(), 0);
        return 1;
    }
}
