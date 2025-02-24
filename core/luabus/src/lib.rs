#![allow(non_snake_case)]

extern crate lua;
extern crate libc;
extern crate luakit;

mod dns;

use lua::lua_State;
use libc::c_int as int;

use luakit::{ Luakit, LuaPush };

#[no_mangle]
pub extern "C" fn luaopen_luabus(L: *mut lua_State) -> int {
    let mut kit = Luakit::load(L);
    let mut luabus = kit.new_table(Some("luabus"));
    // luabus.set_function("sleep", timer::timer_sleep);
    luabus.native_to_lua(L)
}
