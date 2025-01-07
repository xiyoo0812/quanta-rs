#![allow(non_snake_case)]

extern crate lua;
extern crate libc;
extern crate luakit;

mod toml;

use libc::c_int as int;
use toml::{ encode, decode };
use lua::{ cstr, lua_reg, lua_State };

#[no_mangle]
pub extern "C" fn luaopen_ltoml(L: *mut lua_State) -> int {
    let LFUNS = [
        lua_reg!("encode", encode),
        lua_reg!("decode", decode),
        lua_reg!(),
    ];
    unsafe {
        lua::lua_createtable(L, LFUNS.len() as i32, 0);
        lua::luaL_setfuncs(L, LFUNS.as_ptr(), 0);
        return 1;
    }
}
