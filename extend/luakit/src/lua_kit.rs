#![allow(non_snake_case)]
#![allow(dead_code)]

use lua::lua_State;
use libc::c_char as char;

use crate::lua_stack::*;
use crate::lua_function::*;
use crate::lua_base::LuaGuard;
use crate::lua_reference::Reference;
use crate::{ lua_call_function_impl, variadic_return_impl };

pub struct Luakit {
    m_L: *mut lua_State,
}

impl Luakit {
    pub fn new() -> Luakit {
        unsafe {
            let L =  lua::luaL_newstate();
            lua::luaL_openlibs(L);
            lua::lua_checkstack(L, 1024);
            Luakit { m_L: L }
        }
    }

    pub fn load(L: *mut lua_State) -> Luakit {
        Luakit { m_L: L }
    }

    pub fn L(&mut self) -> *mut lua_State {
        return self.m_L;
    }

    // pub fn set<V>(&mut self, name: &str, value: V) {
    //     native_to_lua(self.m_L, value);
    //     unsafe { lua::lua_setglobal(self.m_L, cstr(name)); }
    // }

    // pub fn get<V>(&mut self, name: &str) -> V {
    //     let _ = LuaGuard::new(self.m_L);
    //     unsafe { lua::lua_getglobal(self.m_L, cstr(name)); }
    //     return lua_to_native<V>(self.m_L, -1);
    // }
    
    pub unsafe fn run_file(&mut self, file: *const char) ->Result<bool, String> {
        let _ = LuaGuard::new(self.m_L);
        if lua::luaL_loadfile(self.m_L, file) == 1 {
            let err= lua::lua_tostring(self.m_L, -1).unwrap();
            print!("lua loadfile err: {}", err);
            return Err(err);
        }
        return lua_call_function(self.m_L, 0, 0);
    }

    pub unsafe fn run_script(&mut self, script: *const char) ->Result<bool, String> {
        let _ = LuaGuard::new(self.m_L);
        if lua::luaL_loadstring(self.m_L, script) == 1 {
            let err= lua::lua_tostring(self.m_L, -1).unwrap();
            print!("lua loadstring err: {}", err);
            return Err(err);
        }
        return lua_call_function(self.m_L, 0, 0);
    }

    pub unsafe fn call(&mut self) ->Result<bool, String> {
        return lua_call_function(self.m_L, 0, 0);
    }

    pub unsafe fn call_function(&mut self, func: *const char) ->Result<bool, String> {
        return call_global_function(self.m_L, func, 0, 0);
    }
    
    lua_call_function_impl!(call_lua, );
    lua_call_function_impl!(call_lua1, A);
    lua_call_function_impl!(call_lua2, A, B);
    lua_call_function_impl!(call_lua3, A, B, C);
    lua_call_function_impl!(call_lua4, A, B, C, D);
    lua_call_function_impl!(call_lua5, A, B, C, D, E);
    lua_call_function_impl!(call_lua6, A, B, C, D, E, F);
    lua_call_function_impl!(call_lua7, A, B, C, D, E, F, G);
    lua_call_function_impl!(call_lua8, A, B, C, D, E, F, G, H);
    lua_call_function_impl!(call_lua9, A, B, C, D, E, F, G, H, I);
    lua_call_function_impl!(call_lua10, A, B, C, D, E, F, G, H, I, J);

    variadic_return_impl!(variadic_return1, A);
    variadic_return_impl!(variadic_return2, A, B);
    variadic_return_impl!(variadic_return3, A, B, C);
    variadic_return_impl!(variadic_return4, A, B, C, D);
    variadic_return_impl!(variadic_return5, A, B, C, D, E);
    variadic_return_impl!(variadic_return6, A, B, C, D, E, F);
    variadic_return_impl!(variadic_return7, A, B, C, D, E, F, G);
    variadic_return_impl!(variadic_return8, A, B, C, D, E, F, G, H);
    variadic_return_impl!(variadic_return9, A, B, C, D, E, F, G, H, I);
    variadic_return_impl!(variadic_return10, A, B, C, D, E, F, G, H, I, J);
}

impl Drop for Luakit {
    fn drop(&mut self) {
        unsafe { lua::lua_close(self.m_L); }
    }
}

