#![allow(non_snake_case)]

extern crate lua;
extern crate libc;
extern crate luakit;

mod json;

use lua::lua_State;
use libc::c_int as int;

use luakit::{ Luakit, LuaPush };

#[no_mangle]
pub extern "C" fn luaopen_ljson(L: *mut lua_State) -> int {
    let mut kit = Luakit::load(L);
    let mut ljson = kit.new_table(Some("json"));
    ljson.set_function("encode", json::encode_impl);
    ljson.set_function("decode", json::decode_impl);
    ljson.native_to_lua(L)
}
