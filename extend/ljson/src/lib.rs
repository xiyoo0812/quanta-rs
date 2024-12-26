#![allow(non_snake_case)]

extern crate libc;
extern crate lua;

#[macro_use]
mod macros;

mod json;

use lua::lua_State;
use libc::c_int as int;

extern "C" fn testf(L: *mut lua_State) -> int {
    unsafe {
        let a = lua::lua_tointeger(L, 1);
        let b = lua::lua_tointeger(L, 2);
        lua::lua_pushinteger(L, a + b);
        return 1;
    }
}

#[no_mangle]
pub extern "C" fn luaopen_ljson(L: *mut lua_State) -> int {
    unsafe{
        lua::lua_createtable(L, 4, 0);
        lua::lua_pushcfunction(L, testf);
        lua::lua_setfield(L, -2, cstr!("test"));
        return 1;
    }
}
