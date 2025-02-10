#![allow(non_snake_case)]

extern crate lua;
extern crate libc;
extern crate luakit;

mod toml;

use lua::lua_State;
use libc::c_int as int;

use luakit::{ Luakit, LuaPush };

#[no_mangle]
pub extern "C" fn luaopen_ltoml(L: *mut lua_State) -> int {
    let mut kit = Luakit::load(L);
    let mut ltoml = kit.new_table(Some("toml"));
    ltoml.set_function("open", toml::open);
    ltoml.set_function("save", toml::save);
    ltoml.set_function("encode", toml::encode);
    ltoml.set_function("decode", toml::decode);
    ltoml.native_to_lua(L)
}
