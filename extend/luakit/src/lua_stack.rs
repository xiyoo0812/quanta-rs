#![allow(non_snake_case)]
#![allow(dead_code)]

use lua::lua_State;
use libc::c_char as char;
use std::collections::HashMap;

pub trait LuaPush {
    fn native_to_lua(self, L: *mut lua_State) -> i32;
}

pub trait LuaRead: Sized {
    fn lua_to_native(L: *mut lua_State, index: i32) -> Option<Self>;
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
        if !lua::lua_isboolean(L, index) {
            return Some(false)
        }
        Some(lua::lua_toboolean(L, index))
    }
}

impl LuaPush for *const char {
    fn native_to_lua(self, L: *mut lua_State) -> i32 {
        unsafe { lua::lua_pushstring(L, self); };
        1
    }
}

impl LuaPush for String {
    fn native_to_lua(self, L: *mut lua_State) -> i32 {
        unsafe { lua::lua_pushlstring(L, self.as_ptr() as *const i8, self.len() as usize); };
        1
    }
}


impl LuaRead for String {
    fn lua_to_native(L: *mut lua_State, index: i32) -> Option<String> {
        lua::lua_tolstring(L, index)
    }
}

impl<'s> LuaPush for &'s str {
    fn native_to_lua(self, L: *mut lua_State) -> i32 {
        unsafe { lua::lua_pushlstring(L, self.as_ptr() as *const i8, self.len() as usize); };
        1
    }
}

impl<T> LuaPush for Vec<T> where T: LuaPush {
    fn native_to_lua(self, L: *mut lua_State) -> i32 {
        unsafe {
            lua::lua_createtable(L, self.len() as i32, 0);
            for (i, item) in self.into_iter().enumerate() {
                item.native_to_lua(L);
                lua::lua_rawseti(L, -2, (i + 1) as i32);
            }
            1
        }
    }
}

impl<T> LuaRead for Vec<T> where T: LuaRead {
    fn lua_to_native(L: *mut lua_State, index: i32) -> Option<Vec<T>> {
        let mut vec = Vec::new();
        unsafe {
            let len = lua::lua_rawlen(L, index);
            for i in 1..len + 1 {
                lua::lua_rawgeti(L, index, i as i32);
                vec.push(T::lua_to_native(L, i as i32).unwrap());
                lua::lua_pop(L, 1);
            }
        }
        Some(vec)
    }
}

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
            lua::lua_pushnil(L);
            while lua::lua_next(L, index) != 0 {
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