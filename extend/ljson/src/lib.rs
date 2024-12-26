extern crate libc;
extern crate lua;

use lua::*;
use libc::c_int;

extern "C" fn testf(l: *mut lua_State) -> c_int {
    unsafe {
        let a = lua_tointeger(l, 1);
        let b = lua_tointeger(l, 2);
        lua_pushinteger(l, a + b);
        return 1;
    }
}

unsafe fn regitser_func(l: *mut lua_State, name: &str, f: lua_CFunction) {
    lua_pushcfunction(l, f);
    luar_setfield(l, -2, name);
}

#[no_mangle]
pub extern "C" fn luaopen_ljson(l: *mut lua_State) -> c_int {
    unsafe{
        lua_createtable(l, 4, 0);
        regitser_func(l, "test", testf);
        return 1;
    }
}
