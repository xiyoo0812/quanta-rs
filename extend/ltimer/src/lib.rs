#![allow(non_snake_case)]

extern crate lua;
extern crate libc;
extern crate luakit;

mod timer;

use timer::*;
use libc::c_int as int;
use lua::{ cstr, lua_reg, lua_State };

use luakit::LuaPush;

#[no_mangle]
pub extern "C" fn luaopen_ltimer(L: *mut lua_State) -> int {
    let LFUNS = [
        lua_reg!("sleep", timer_sleep),
        lua_reg!("insert", timer_insert),
        lua_reg!("update", timer_update),
        lua_reg!("now", |L| { luakit::variadic_return!(L, luakit::now()) }),
        lua_reg!("now_ms", |L| { luakit::variadic_return!(L, luakit::now_ms()) }),
        lua_reg!("now_ns", |L| { luakit::variadic_return!(L, luakit::now_ns()) }),
        lua_reg!("steady", |L| { luakit::variadic_return!(L, luakit::steady()) }),
        lua_reg!("steady_ms", |L| { luakit::variadic_return!(L, luakit::steady_ms()) }),
        lua_reg!("time", |L| { luakit::variadic_return!(L, luakit::now_ms(), luakit::steady_ms()) }),
        lua_reg!(),
    ];
    unsafe {
        lua::lua_createtable(L, LFUNS.len() as i32, 0);
        lua::luaL_setfuncs(L, LFUNS.as_ptr(), 0);
        return 1;
    }
}
