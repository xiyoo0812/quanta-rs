#![allow(non_snake_case)]
#![allow(dead_code)]

use libc::c_char as char;
use libc::c_void as void;
use lua::{cstr, to_char, lua_State};

use std::thread::ThreadId;
use std::collections::HashMap;

use crate::lua_get_meta_name;
use crate::lua_base::LuaGuard;

pub trait LuaPush {
    fn native_to_lua(self, L: *mut lua_State) -> i32;
}

pub trait LuaPushFn {
    fn native_to_lua(self, L: *mut lua_State) -> i32;
}

pub trait LuaPushFnMut {
    fn native_to_lua(self, L: *mut lua_State) -> i32;
}

pub trait LuaPushLuaFn {
    fn native_to_lua(self, L: *mut lua_State) -> i32;
}

pub trait LuaPushLuaFnMut {
    fn native_to_lua(self, L: *mut lua_State) -> i32;
}

pub trait LuaRead: Sized {
    fn lua_to_native(L: *mut lua_State, index: i32) -> Option<Self>;
}


pub trait LuaGc {
    fn __gc(&mut self) -> bool { true }
}

impl <T> LuaPush for Box<T> {
    fn native_to_lua(self, L: *mut lua_State) -> i32 {
        let tptr: *mut T = Box::into_raw(self);
        lua_push_userdata(L, tptr)
    }
}

impl<T> LuaRead for Box<T> {
    fn lua_to_native(L: *mut lua_State, index: i32) -> Option<Box<T>> {
        if !lua::lua_isuserdata(L, index) {
            return None
        }
        let pvoid = lua_load_userdata(L, index);
        unsafe { Some(Box::from_raw(pvoid as *mut T)) }
    }
}

impl LuaRead for *mut void {
    fn lua_to_native(L: *mut lua_State, index: i32) -> Option<*mut void> {
        if !lua::lua_isuserdata(L, index) {
            return None
        }
        Some(lua_load_userdata(L, index))
    }
}

impl LuaPush for lua::LuaNil {
    fn native_to_lua(self, L: *mut lua_State) -> i32 {
        unsafe { lua::lua_pushnil(L) };
        1
    }
}

impl LuaPush for bool {
    fn native_to_lua(self, L: *mut lua_State) -> i32 {
        unsafe { lua::lua_pushboolean(L, self.clone() as libc::c_int) };
        1
    }
}

impl LuaRead for bool {
    fn lua_to_native(L: *mut lua_State, index: i32) -> Option<bool> {
        Some(lua::lua_toboolean(L, index))
    }
}

impl LuaPush for *const char {
    fn native_to_lua(self, L: *mut lua_State) -> i32 {
        unsafe { lua::lua_pushstring_(L, self); };
        1
    }
}

impl LuaPush for Vec<u8>{
    fn native_to_lua(self, L: *mut lua_State) -> i32 {
        let buf = self.as_ptr() as *const i8;
        unsafe { lua::lua_pushlstring_(L, buf,  self.len()) };
        1
    }
}

impl LuaRead for Vec<u8>{
    fn lua_to_native(L: *mut lua_State, index: i32) -> Option<Vec<u8>> {
        let mut size: usize = 0;
        let cstr = unsafe { lua::lua_tolstring_(L, index, &mut size) };
        let bytes = unsafe { std::slice::from_raw_parts(cstr as *const u8, size) };
        Some(bytes.to_vec())
    }
}

impl<T> LuaPush for Option<T> where T: LuaPush {
    fn native_to_lua(self, L: *mut lua_State) -> i32 {
        if let Some(val) = self {
            val.native_to_lua(L)
        } else {
            unsafe { lua::lua_pushnil(L) };
            1
        }
    }
}

impl LuaPush for &[u8]{
    fn native_to_lua(self, L: *mut lua_State) -> i32 {
        let buf = self.as_ptr() as *const i8;
        unsafe { lua::lua_pushlstring_(L, buf,  self.len()) };
        1
    }
}

impl<'a> LuaRead for &'a [u8]{
    fn lua_to_native(L: *mut lua_State, index: i32) -> Option<&'a [u8]> {
        let mut size: usize = 0;
        let cstr = unsafe { lua::lua_tolstring_(L, index, &mut size) };
        let bytes = unsafe { std::slice::from_raw_parts(cstr as *const u8, size) };
        Some(bytes)
    }
}

impl LuaPush for String {
    fn native_to_lua(self, L: *mut lua_State) -> i32 {
        unsafe { lua::lua_pushlstring_(L, to_char!(self), self.len() as usize); };
        1
    }
}

impl LuaRead for String {
    fn lua_to_native(L: *mut lua_State, index: i32) -> Option<String> {
        let bytes = lua::lua_tolstring(L, index);
        match String::from_utf8(bytes.to_vec()) {
            Ok(str) => Some(str),
            Err(_) => None
        }
    }
}

impl LuaPush for ThreadId {
    //临时先这样，后面等ThreadId支持au_u64，再修改
    fn native_to_lua(self, L: *mut lua_State) -> i32 {
        unsafe { lua::lua_pushinteger(L, &self as *const ThreadId as isize); };
        1
    }
}

impl<'s> LuaPush for &'s str {
    fn native_to_lua(self, L: *mut lua_State) -> i32 {
        unsafe { lua::lua_pushlstring_(L, to_char!(self), self.len() as usize); };
        1
    }
}

// 等 rust 特化特性放开泛型
// impl<T> LuaPush for Vec<T> where T: LuaPush {
//     fn native_to_lua(self, L: *mut lua_State) -> i32 {
//         unsafe {
//             lua::lua_createtable(L, self.len() as i32, 0);
//             for (i, item) in self.into_iter().enumerate() {
//                 item.native_to_lua(L);
//                 lua::lua_rawseti(L, -2, (i + 1) as i32);
//             }
//             1
//         }
//     }
// }

// impl<T> LuaRead for Vec<T> where T: LuaRead {
//     fn lua_to_native(L: *mut lua_State, index: i32) -> Option<Vec<T>> {
//         let mut vec = Vec::new();
//         unsafe {
//             let len = lua::lua_rawlen(L, index);
//             for i in 1..len + 1 {
//                 lua::lua_rawgeti(L, index, i as i32);
//                 vec.push(T::lua_to_native(L, i as i32).unwrap());
//                 lua::lua_pop(L, 1);
//             }
//         }
//         Some(vec)
//     }
// }

impl<K, V> LuaPush for HashMap<K, V> where K: LuaPush, V: LuaPush {
    fn native_to_lua(self, L: *mut lua_State) -> i32 {
        unsafe {
            lua::lua_createtable(L, 0, self.len() as i32);
            for (key, value) in self.into_iter() {
                key.native_to_lua(L);
                value.native_to_lua(L);
                lua::lua_settable(L, -3);
            }
            1
        }
    }
}

impl<K, V> LuaRead for HashMap<K, V> where K: LuaRead + Eq + std::hash::Hash, V: LuaRead {
    fn lua_to_native(L: *mut lua_State, index: i32) -> Option<HashMap<K, V>> {
        let mut map = HashMap::new();
        unsafe {
            let i = lua::lua_absindex(L, index);
            lua::lua_pushnil(L);
            while lua::lua_next(L, i) != 0 {
                let key = K::lua_to_native(L, -2).unwrap();
                let val = V::lua_to_native(L, -1).unwrap();
                map.insert(key, val);
                lua::lua_pop(L, 1);
            }
        }
        Some(map)
    }
}

macro_rules! integer_impl(
    ($t:ident) => (
        impl LuaPush for $t {
            fn native_to_lua(self, L: *mut lua_State) -> i32 {
                unsafe { lua::lua_pushinteger(L, self as lua::lua_Integer) };
                1
            }
        }
        impl LuaRead for $t {
            fn lua_to_native(L: *mut lua_State, index: i32) -> Option<$t> {
                let mut success = 0;
                let val = unsafe { lua::lua_tointegerx(L, index, &mut success) };
                match success {
                    0 => Some(0),
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

macro_rules! numeric_impl(
    ($t:ident) => (
        impl LuaPush for $t {
            fn native_to_lua(self, L: *mut lua_State) -> i32 {
                unsafe { lua::lua_pushnumber(L, self as lua::lua_Number) };
                1
            }
        }
        impl LuaRead for $t {
            fn lua_to_native(L: *mut lua_State, index: i32) -> Option<$t> {
                let mut success = 0;
                let val = unsafe { lua::lua_tonumberx(L, index, &mut success) };
                match success {
                    0 => Some(0.0),
                    _ => Some(val as $t)
                }
            }
        }
    );
);

numeric_impl!(f32);
numeric_impl!(f64);

macro_rules! tuple_impl {
    () => {
        impl LuaPush for () {
            fn native_to_lua(self, L: *mut lua_State) -> i32 {
                unsafe { lua::lua_pushnil(L) };
                1
            }
        }
        impl LuaRead for () {
            fn lua_to_native(_: *mut lua_State, _: i32) -> Option<Self> {
                Some(())
            }
        }
    };
    ($first:ident, $($other:ident),*) => (
        #[allow(unused_mut)]
        #[allow(non_snake_case)]
        impl<$first: LuaPush, $($other: LuaPush),*> LuaPush for ($first, $($other),*) {
            fn native_to_lua(self, L: *mut lua_State) -> i32 {
                match self {
                    ($first, $($other),*) => {
                        let mut total = $first.native_to_lua(L);
                        $(total += $other.native_to_lua(L); )*
                        total
                    }
                }
            }
        }

        #[allow(non_snake_case)]
        #[allow(unused_assignments)]
        impl<$first: LuaRead, $($other: LuaRead),*> LuaRead for ($first, $($other),*) {
            fn lua_to_native(L: *mut lua_State, index: i32) -> Option<($first, $($other),*)> {
                let mut i = index;
                let $first: $first = match LuaRead::lua_to_native(L, i) {
                    Some(v) => v,
                    None => return None
                };
                i += 1;
                $(
                    let $other: $other = match LuaRead::lua_to_native(L, i) {
                        Some(v) => v,
                        None => return None
                    };
                    i += 1;
                )*
                Some(($first, $($other),*))
            }
        }
    );
}

tuple_impl!();
tuple_impl!(A, );
tuple_impl!(A, B);
tuple_impl!(A, B, C);
tuple_impl!(A, B, C, D);
tuple_impl!(A, B, C, D, E);
tuple_impl!(A, B, C, D, E, F);
tuple_impl!(A, B, C, D, E, F, G);
tuple_impl!(A, B, C, D, E, F, G, H);
tuple_impl!(A, B, C, D, E, F, G, H, I);
tuple_impl!(A, B, C, D, E, F, G, H, I, J);
tuple_impl!(A, B, C, D, E, F, G, H, I, J, K);
tuple_impl!(A, B, C, D, E, F, G, H, I, J, K, L);
tuple_impl!(A, B, C, D, E, F, G, H, I, J, K, L, M);

pub fn lua_load_userdata(L : *mut lua_State, index: i32) -> *mut void  {
    unsafe {
        if lua::lua_istable(L, index) {
            let _gl = LuaGuard::new(L);
            lua::lua_getfield(L, index, cstr!("__pointer__"));
            if lua::lua_isnil(L, -1) {
                return std::ptr::null_mut();
            }
            return lua::lua_touserdata(L, -1);
        }
        if lua::lua_isuserdata(L, index) {
            return lua::lua_touserdata(L, index);
        }
        std::ptr::null_mut()
    }
}

pub fn lua_push_userdata<T>(L : *mut lua_State, obj: *mut T) -> i32  {
    unsafe {
        //__objects__
        lua::lua_getfield(L, lua::LUA_REGISTRYINDEX, cstr!("__objects__"));
        if lua::lua_isnil(L, -1) {
            lua::lua_pop(L, 1);
            lua::lua_createtable(L, 0, 128);
            lua::lua_createtable(L, 0, 4);
            lua::lua_pushstring(L, "v");
            lua::lua_setfield(L, -2, cstr!("__mode"));
            lua::lua_setmetatable(L, -2);
            lua::lua_pushvalue(L, -1);
            lua::lua_setfield(L, lua::LUA_REGISTRYINDEX, cstr!("__objects__"));
        }
        // stack: __objects__
        let pkey = obj as isize;
        if lua::lua_geti(L, -1, pkey) != lua::LUA_TTABLE {
            lua::lua_pop(L, 1);
            lua::lua_createtable(L, 0, 4);
            lua::lua_pushlightuserdata(L, obj as *mut void);
            lua::lua_setfield(L, -2, cstr!("__pointer__"));
            // stack: __objects__, table
            let meta_name = lua_get_meta_name::<T>();
            lua::luaL_getmetatable(L, &meta_name);
            if lua::lua_isnil(L, -1) {
                lua::lua_pop(L, 3);
                lua::lua_pushlightuserdata(L, obj as *mut void);
                return 1;
            }
            // stack: __objects__, table, metatab
            lua::lua_setmetatable(L, -2);
            lua::lua_pushvalue(L, -1);
            // stack: __objects__, table, table
            lua::lua_seti(L, -3, pkey);
        }
        // stack: __objects__, table
        lua::lua_remove(L, -2);
        1
    }
}

