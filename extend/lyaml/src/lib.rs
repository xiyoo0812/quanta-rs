#![allow(non_snake_case)]

extern crate lua;
extern crate libc;
extern crate luakit;

mod yaml;

use libc::c_int as int;
use lua::{ cstr, lua_reg, lua_State };
use yaml::{ encode, decode, open, save };

#[no_mangle]
pub extern "C" fn luaopen_lyaml(L: *mut lua_State) -> int {
    let LFUNS = [
        lua_reg!("open", open),
        lua_reg!("save", save),
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
