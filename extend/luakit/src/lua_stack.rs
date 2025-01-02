#![allow(non_snake_case)]
#![allow(dead_code)]

use std::ffi::CString;

use libc::c_int as int;
use lua::{ cstr, lua_State };

pub trait LuaPush {
    fn native_to_lua(self, lua: *mut lua_State) -> i32;
}

pub trait LuaRead: Sized {
    fn lua_to_native(lua: *mut lua_State, index: i32) -> Option<Self>;
}

impl LuaPush for bool {
    fn native_to_lua(self, lua: *mut lua_State) -> i32 {
        unsafe { lua::lua_pushboolean(lua, self.clone() as libc::c_int) };
        1
    }
}

impl LuaRead for bool {
    fn lua_to_native(lua: *mut lua_State, index: i32, _pop: i32) -> Option<bool> {
        if unsafe { lua::lua_isboolean(lua, index) } != true {
            return None;
        }
        Some(unsafe { lua::lua_toboolean(lua, index) != 0 })
    }
}

impl LuaPush for String {
    fn native_to_lua(self, lua: *mut lua_State) -> i32 {
        if let Some(value) = CString::new(&self[..]).ok() {
            unsafe { lua::lua_pushstring(lua, value.as_ptr()) };
            1
        } else {
            let value = CString::new(&"UNVAILED STRING"[..]).unwrap();
            unsafe { lua::lua_pushstring(lua, value.as_ptr()) };
            1
        }
    }
}

impl LuaRead for String {
    fn lua_to_native(lua: *mut lua_State, index: i32, _pop: i32) -> Option<String> {
        return unsafe { lua::lua_tolstring(lua, index) };
    }
}

impl<'s> LuaPush for &'s str {
    fn native_to_lua(self, lua: *mut lua_State) -> i32 {
        if let Some(value) = CString::new(&self[..]).ok() {
            unsafe { lua::lua_pushstring(lua, value.as_ptr()) };
            1
        } else {
            let value = CString::new(&"UNVAILED STRING"[..]).unwrap();
            unsafe { lua::lua_pushstring(lua, value.as_ptr()) };
            1
        }
    }
}

macro_rules! integer_impl(
    ($t:ident) => (
        impl LuaPush for $t {
            fn native_to_lua(self, lua: *mut lua_State) -> i32 {
                unsafe { lua::lua_pushinteger(lua, self as lua::lua_Integer) };
                1
            }
        }
        impl LuaRead for $t {
            fn lua_to_native(lua: *mut lua_State, index: i32) -> Option<$t> {
                let mut success = 0;
                let val = unsafe { lua::lua_tointegerx(lua, index, &mut success) };
                match success {
                    0 => None,
                    _ => Some(val as $t)
                }
            }
        }
    );
);

macro_rules! numeric_impl(
    ($t:ident) => (
        impl LuaPush for $t {
            fn native_to_lua(self, lua: *mut lua_State) -> i32 {
                unsafe { lua::lua_pushnumber(lua, self as lua::lua_Integer) };
                1
            }
        }
        impl LuaRead for $t {
            fn lua_to_native(lua: *mut lua_State, index: i32) -> Option<$t> {
                let mut success = 0;
                let val = unsafe { lua::lua_tonumberx(lua, index, &mut success) };
                match success {
                    0 => None,
                    _ => Some(val as $t)
                }
            }
        }
    );
);


integer_impl!(i8);
integer_impl!(u8);
integer_impl!(i16);
integer_impl!(u16);
integer_impl!(i32);
integer_impl!(u32);
integer_impl!(i64);
integer_impl!(u64);
integer_impl!(usize);

numeric_impl!(f32);
numeric_impl!(f64);