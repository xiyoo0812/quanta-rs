extern crate libc;
extern crate lua;

use lua::*;
use libc::c_int;
use std::ffi::{CString};

extern "C" fn testf(l: *mut lua_State) -> c_int {
    unsafe {
        let mut isnum = 0;
        let a = lua_tointegerx(l, 1, &mut isnum);
        let b = lua_tointegerx(l, 2, &mut isnum);
        lua_pushinteger(l, a+b);
        return 1;
    }
}

fn regitser_func(l: *mut lua_State, name: &str, f: lua_CFunction) {
    unsafe {
        let fname = CString::new(name).unwrap();
        lua_pushcfunction(l, f);
        lua_setfield(l, -2, fname.as_ptr());
    }
}

#[no_mangle]
pub extern "C" fn luaopen_ljson(l: *mut lua_State) -> c_int {
    unsafe{
        lua_createtable(l, 4, 0);
        regitser_func(l, "test", testf);
        return 1;
    }
}
