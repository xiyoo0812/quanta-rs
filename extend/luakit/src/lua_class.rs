#![allow(non_snake_case)]
#![allow(dead_code)]

use lua::lua_State;
use libc::c_int as int;
use libc::c_char as char;

use std::{ mem, ptr };

use crate::lua_get_meta_name;
use crate::lua_stack::{ LuaPush, LuaRead };

pub struct MemberWrapper<F> {
    pub getter: Option<F>,
    pub setter: Option<F>,
    pub is_function : bool,
}

impl<F> LuaPush for MemberWrapper<F> where F: LuaPush  {
    fn native_to_lua(self, L: *mut lua_State) -> i32 {
        unsafe {
            let lua_data = lua::lua_newuserdata(L, mem::size_of::<Self>() as libc::size_t);
            let lua_data: *mut F = std::mem::transmute(lua_data);
            ptr::write(lua_data as *mut _, self);
        }
        1
    }
}

#[macro_export]
macro_rules! lua_wrapper_member_impl {
    ($name:ident, $($p:ident),*) => (
        pub fn $name<Z>(&mut self, mname: *const char, f: Z) where Z: LuaPush {
            let wrapper = FuncWrapper { function: f, marker: PhantomData };
            self.set(fname, wrapper);
        }
    )
}

pub fn lua_class_index<T>(L: *mut lua_State) -> int where T: LuaRead  {
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