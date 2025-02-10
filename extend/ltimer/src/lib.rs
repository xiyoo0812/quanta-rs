#![allow(non_snake_case)]

extern crate lua;
extern crate libc;
extern crate luakit;

mod timer;

use lua::lua_State;
use libc::c_int as int;

use luakit::{ Luakit, LuaPush };

#[no_mangle]
pub extern "C" fn luaopen_ltimer(L: *mut lua_State) -> int {
    let mut kit = Luakit::load(L);
    let mut ltimer = kit.new_table(Some("timer"));
    ltimer.set_function("sleep", timer::timer_sleep);
    ltimer.set_function("insert", timer::timer_insert);
    ltimer.set_function("update", timer::timer_update);
    ltimer.set_function("now", |L| { luakit::variadic_return!(L, luakit::now()) });
    ltimer.set_function("clock", |L| { luakit::variadic_return!(L, luakit::steady()) });
    ltimer.set_function("now_ms", |L| { luakit::variadic_return!(L, luakit::now_ms()) });
    ltimer.set_function("now_ns", |L| { luakit::variadic_return!(L, luakit::now_ns()) });
    ltimer.set_function("clock_ms", |L| { luakit::variadic_return!(L, luakit::steady_ms()) });
    ltimer.set_function("time", |L| { luakit::variadic_return!(L, luakit::now_ms(), luakit::steady_ms()) });
    ltimer.native_to_lua(L)
}
