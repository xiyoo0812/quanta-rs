#![allow(non_snake_case)]

extern crate lua;
extern crate libc;

mod json;

use libc::c_int as int;
use lua::{ cstr, lreg, lreg_null, lua_State };

extern "C" fn encode(L: *mut lua_State) -> int {
    unsafe { return json::encode_impl(L); }
}

extern "C" fn decode(L: *mut lua_State) -> int {
    unsafe { return json::decode_impl(L); }
}

#[no_mangle]
pub extern "C" fn luaopen_ljson(L: *mut lua_State) -> int {
        let l = [
            lreg!("decode", decode),
            lreg!("encode", encode),
            lreg_null!(),
        ];
    unsafe {
        lua::lua_createtable(L, 2, 0);
        lua::luaL_setfuncs(L, l.as_ptr(), 0);
        return 1;
    }
}
