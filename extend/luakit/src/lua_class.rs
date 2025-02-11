#![allow(non_snake_case)]
#![allow(dead_code)]

use libc::c_int as int;
use lua::{cstr, to_char, to_cptr, lua_State };

use crate::lua_stack::LuaGc;
use crate::{lua_get_meta_name, lua_load_userdata};

pub fn lua_class_index<T>(L: *mut lua_State) -> int {
    unsafe {
        let objptr = lua_load_userdata(L, 1);
        let key = lua::lua_tostring(L, 2);
        if objptr.is_null() || key.is_empty() {
            lua::lua_pushnil(L);
            return 1;
        }
        let meta_name = lua_get_meta_name::<T>();
        lua::luaL_getmetatable(L, to_char!(meta_name));
        if lua::lua_istable(L, -1) {
            lua::lua_getfield(L, -1, to_cptr(key));
            if lua::lua_istable(L, -1) {
                lua::lua_getfield(L, -1, cstr!("__getter"));
                if lua::lua_isfunction(L, -1) {
                    lua::lua_getfield(L, -2, cstr!("__wrapper"));
                    lua::lua_pushlightuserdata(L, objptr);
                    lua::lua_call(L, 2, 1);
                    return 1;
                }
            }
        }
        lua::lua_pushnil(L);
        1
    }
}

pub fn lua_class_newindex<T>(L: *mut lua_State) -> int {
    unsafe {
        let objptr = lua_load_userdata(L, 1);
        let key = lua::lua_tostring(L, 2);
        if objptr.is_null() || key.is_empty() {
            return 0;
        }
        let meta_name = lua_get_meta_name::<T>();
        lua::luaL_getmetatable(L, to_char!(meta_name));
        if lua::lua_istable(L, -1) {
            lua::lua_getfield(L, -1, to_cptr(key));
            if lua::lua_isnil(L, -1) {
                //没有找到成员虚表信息直接写
                lua::lua_pop(L, 2);
                lua::lua_rawset(L, -3);
                return 0;
            }
            if lua::lua_istable(L, -1) {
                lua::lua_getfield(L, -1, cstr!("__setter"));
                if lua::lua_isfunction(L, -1) {
                    lua::lua_getfield(L, -2, cstr!("__wrapper"));
                    lua::lua_pushlightuserdata(L, objptr);
                    lua::lua_pushvalue(L, 3);
                    lua::lua_call(L, 3, 0);
                }
            }
        }
        0
    }
}

pub fn lua_class_gc<T>(L: *mut lua_State) -> int where T: LuaGc {
    let objptr = lua_load_userdata(L, 1);
    unsafe {
        if !objptr.is_null() {
            let obj : *mut T = std::mem::transmute(objptr);
            if (&mut *obj).__gc() {
                let _ = Box::from_raw(obj);
            }
        }
        0
    }
}

#[macro_export]
macro_rules! new_class {
    ($class:ty, $lua:expr, $name:expr $(, $fname:expr, $func:expr)* $(;$mname:expr => $member:ident: $mt: ident)*) => ({
        let L = $lua.L();
        let _gl = luakit::LuaGuard::new(L);
        let meta_name = luakit::lua_get_meta_name::<$class>();
        lua::luaL_getmetatable(L, lua::to_char!(meta_name));
        if (lua::lua_isnil(L, -1)) {
            lua::lua_pop(L, 1);
            let meta = [
                lua::lua_reg!("__gc", luakit::lua_class_gc::<$class>),
                lua::lua_reg!("__index", luakit::lua_class_index::<$class>),
                lua::lua_reg!("__newindex", luakit::lua_class_newindex::<$class>),
                lua::lua_reg!(),
            ];
            unsafe {
                lua::luaL_newmetatable(L, lua::to_char!(meta_name));
                lua::luaL_setfuncs(L, meta.as_ptr(), 0);
                $(
                    $fname.native_to_lua(L);
                    let wrapper = luakit::ClassFuncWrapper { function: $func, marker: std::marker::PhantomData };
                    wrapper.native_to_lua(L);
                    lua::lua_rawset(L, -3);
                )*
                $(
                    $mname.native_to_lua(L);
                    let offset = std::mem::offset_of!($class, $member);
                    let wrapper = luakit::ClassMemberWrapper::<$class, $mt>{ offset: offset, marker: std::marker::PhantomData };
                    wrapper.native_to_lua(L);
                    lua::lua_rawset(L, -3);
                )*
            }
        }
    })
}

#[macro_export]
macro_rules! new_enum {
    ($lua:expr, $name:expr, $($key:expr, $val:expr), +) => ({
        let mut table = $lua.new_table(Some($name));
        $(table.set($key, $val as i32);)+
    });
}
