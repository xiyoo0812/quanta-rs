#![allow(non_snake_case)]
#![allow(dead_code)]

use lua::lua_State;
use libc::c_int as int;
use libc::c_char as char;

use crate::lua_get_meta_name;
use crate::lua_stack::{ LuaPush, LuaRead };

pub fn lua_class_index<T>(L: *mut lua_State) -> int where T: LuaRead  {
    let res = T::lua_to_native(L, 1);
    if res.is_none() {
        return 0;
    }
    0
}

pub fn lua_class_newindex<T>(L: *mut lua_State) -> int where T: LuaRead {
    let res = T::lua_to_native(L, 1);
    if res.is_none() {
        return 0;
    }
    let key = lua::lua_tostring(L, 2).unwrap();
    let meta_name = lua_get_meta_name::<T>();
    unsafe {
        lua::luaL_getmetatable(L, meta_name.as_ptr() as *const char);
        lua::lua_pushstring(L, key.as_ptr() as *const char);
        lua::lua_rawget(L, -2);
    }
    0
}

pub fn lua_class_gc<T>(L: *mut lua_State) -> int where T: LuaRead {
    let res = T::lua_to_native(L, 1);
    match res {
        Some(v) => {
            if T::__gc() {
                drop(v);
            }
        },
        None => {}
    }
    0
}

#[macro_export]
macro_rules! new_class {
    ($class:ty, $lua:expr, $name:expr$(,$key:expr, $member:expr)*) => ({
        let L = $lua.L();
        let _gl = luakit::LuaGuard::new(L);
        let meta_name = luakit::lua_get_meta_name::<$class>();
        lua::luaL_getmetatable(L, meta_name.as_ptr() as *const char);
        if (lua::lua_isnil(L, -1)) {
            lua::lua_pop(L, 1);
            let meta = [
                lua::lua_reg!("__gc", luakit::lua_class_gc::<$class>),
                lua::lua_reg!("__index", luakit::lua_class_index::<$class>),
                lua::lua_reg!("__newindex", luakit::lua_class_newindex::<$class>),
                lua::lua_reg!(),
            ];
            unsafe { 
                lua::luaL_newmetatable(L, meta_name.as_ptr() as *const char);
                lua::luaL_setfuncs(L, meta.as_ptr(), 0);
            }
        }
    })
}

#[macro_export]
macro_rules! new_enum {
    ($lua:expr, $name:expr, $($key:expr, $val:expr), +) => ({
        let mut table = $lua.new_table(Some(cstr!($name)));
        $(
            table.set($key, $val);
        )+
        table
    });
}
