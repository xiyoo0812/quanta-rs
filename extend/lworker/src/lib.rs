#![allow(non_snake_case)]

extern crate lua;
extern crate libc;
extern crate luakit;

mod worker;

use lua::lua_State;
use libc::c_int as int;

use luakit::{ Luakit, LuaPush };

#[no_mangle]
pub extern "C" fn luaopen_lworker(L: *mut lua_State) -> int {
    let mut kit = Luakit::load(L);
    let mut lworker = kit.new_table(Some("worker"));
    lworker.native_to_lua(L)
}
