#![allow(non_snake_case)]

extern crate lua;
extern crate libc;
extern crate luakit;

mod yaml;

use lua::lua_State;
use libc::c_int as int;

use luakit::{ Luakit, LuaPush };

#[no_mangle]
pub extern "C" fn luaopen_lyaml(L: *mut lua_State) -> int {
    let mut kit = Luakit::load(L);
    let mut lyaml = kit.new_table(Some("yaml"));
    lyaml.set_function("open", yaml::open);
    lyaml.set_function("save", yaml::save);
    lyaml.set_function("encode", yaml::encode);
    lyaml.set_function("decode", yaml::decode);
    lyaml.native_to_lua(L)
}
