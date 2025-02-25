#![allow(non_snake_case)]

extern crate lua;
extern crate libc;
extern crate luakit;

mod socket_dns;
mod socket_ping;
mod socket_helper;

use lua::lua_State;
use libc::c_int as int;

use luakit::{ Luakit, LuaPush, LuaPushFn, LuaPushLuaFn };

#[no_mangle]
pub extern "C" fn luaopen_luabus(L: *mut lua_State) -> int {
    let mut kit = Luakit::load(L);
    let mut luabus = kit.new_table(Some("luabus"));
    luakit::set_function!(luabus, "host", socket_dns::gethostip);
    luakit::set_function!(luabus, "dns", socket_dns::gethostbydomain);
    luakit::set_function!(luabus, "ping", socket_ping::socket_ping);
    luakit::set_function!(luabus, "derive_port", socket_helper::derive_port);
    luabus.native_to_lua(L)
}
