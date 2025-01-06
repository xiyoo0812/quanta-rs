#![allow(non_snake_case)]
#![allow(dead_code)]

use std::env;
use std::marker::PhantomData;

use libc::c_char as char;
use lua::{ cstr, lua_State, lua_CFunction };

use crate::lua_stack::*;
use crate::lua_function::*;
use crate::lua_base::LuaGuard;
use crate::lua_table::LuaTable;
use crate::lua_reference::Reference;
use crate::{ lua_wrapper_function_impl, lua_call_function_impl, table_call_function_impl };

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

    pub fn close(&mut self) {
        unsafe { lua::lua_close(self.m_L); }
    }

    pub fn L(&mut self) -> *mut lua_State {
        return self.m_L;
    }

    pub fn push<V>(&mut self, value: V) ->i32 where V: LuaPush {
        return value.native_to_lua(self.m_L);
    }

    pub fn set<V>(&mut self, key: *const char, value: V) where V: LuaPush {
        value.native_to_lua(self.m_L);
        unsafe { lua::lua_setglobal(self.m_L, key); }
    }

    pub fn get<R>(&mut self, key: *const char) -> Option<R> where R: LuaRead {
        let _ = LuaGuard::new(self.m_L);
        unsafe { lua::lua_getglobal(self.m_L, key); }
        return LuaRead::lua_to_native(self.m_L, -1);
    }

    pub fn get_path(&mut self, field: *const char) -> String {
        let _ = LuaGuard::new(self.m_L);
        unsafe {
            lua::lua_getglobal(self.m_L, cstr!("package"));
            lua::lua_getfield(self.m_L, -1, field);
            lua::lua_tostring(self.m_L, -1).unwrap()
        }
    }

    pub fn set_path(&mut self, field: &str, path: &str) {
        if field == "LUA_PATH" {
            self.set_lua_path("path", path);
        } else {
            self.set_lua_path("cpath", path);
        }
    }

    pub fn set_searchers(&mut self, f : lua_CFunction) {
        let _ = LuaGuard::new(self.m_L);
        unsafe {
            lua::lua_getglobal(self.m_L, cstr!("package"));
            lua::lua_getfield(self.m_L, -1, cstr!("searchers"));
            lua::lua_pushcfunction(self.m_L, f);
            lua::lua_rawseti(self.m_L, -2, 2);
        }
    }

    pub fn set_function(&mut self, name: *const char, f : lua_CFunction) {
        let _ = LuaGuard::new(self.m_L);
        unsafe {
            lua::lua_pushcfunction(self.m_L, f);
            lua::lua_setglobal(self.m_L, name);
        }
    }

    pub fn new_table(&mut self, name: Option<*const char>) -> LuaTable {
        let _ = LuaGuard::new(self.m_L);
        unsafe { 
            lua::lua_createtable(self.m_L, 0, 8);
            if !name.is_none() {
                lua::lua_pushvalue(self.m_L, -1);
                lua::lua_setglobal(self.m_L, name.unwrap());
            }
            LuaTable::new(self.m_L)
        }
    }
    
    pub fn run_file(&mut self, file: *const char) ->Result<bool, String> {
        let _ = LuaGuard::new(self.m_L);
        if lua::luaL_loadfile(self.m_L, file) == 1 {
            let err= lua::lua_tostring(self.m_L, -1).unwrap();
            print!("lua loadfile err: {}", err);
            return Err(err);
        }
        return lua_call_function(self.m_L, 0, 0);
    }

    pub fn run_script(&mut self, script: *const char) ->Result<bool, String> {
        let _ = LuaGuard::new(self.m_L);
        unsafe {
            if lua::luaL_loadstring(self.m_L, script) == 1 {
                let err= lua::lua_tostring(self.m_L, -1).unwrap();
                print!("lua loadstring err: {}", err);
                return Err(err);
            }
        }
        return lua_call_function(self.m_L, 0, 0);
    }

    pub fn get_function(&mut self, function: *const char) -> bool {
        return get_global_function(self.m_L, function);
    }

    pub fn call_function(&mut self) ->Result<bool, String> {
        return lua_call_function(self.m_L, 0, 0);
    }

    pub fn call_global(&mut self, func: *const char) ->Result<bool, String> {
        return call_global_function(self.m_L, func, 0, 0);
    }
    
    lua_call_function_impl!(call, );
    lua_call_function_impl!(call1, A);
    lua_call_function_impl!(call2, A, B);
    lua_call_function_impl!(call3, A, B, C);
    lua_call_function_impl!(call4, A, B, C, D);
    lua_call_function_impl!(call5, A, B, C, D, E);
    lua_call_function_impl!(call6, A, B, C, D, E, F);
    lua_call_function_impl!(call7, A, B, C, D, E, F, G);
    lua_call_function_impl!(call8, A, B, C, D, E, F, G, H);
    lua_call_function_impl!(call9, A, B, C, D, E, F, G, H, I);
    lua_call_function_impl!(call10, A, B, C, D, E, F, G, H, I, J);

    table_call_function_impl!(table_call, );
    table_call_function_impl!(table_call1, A);
    table_call_function_impl!(table_call2, A, B);
    table_call_function_impl!(table_call3, A, B, C);
    table_call_function_impl!(table_call4, A, B, C, D);
    table_call_function_impl!(table_call5, A, B, C, D, E);
    table_call_function_impl!(table_call6, A, B, C, D, E, F);
    table_call_function_impl!(table_call7, A, B, C, D, E, F, G);
    table_call_function_impl!(table_call8, A, B, C, D, E, F, G, H);
    table_call_function_impl!(table_call9, A, B, C, D, E, F, G, H, I);
    table_call_function_impl!(table_call10, A, B, C, D, E, F, G, H, I, J);

    lua_wrapper_function_impl!(bind_function,);
    lua_wrapper_function_impl!(bind_function1, A);
    lua_wrapper_function_impl!(bind_function2, A, B);
    lua_wrapper_function_impl!(bind_function3, A, B, C);
    lua_wrapper_function_impl!(bind_function4, A, B, C, D);
    lua_wrapper_function_impl!(bind_function5, A, B, C, D, E);
    lua_wrapper_function_impl!(bind_function6, A, B, C, D, E, F);
    lua_wrapper_function_impl!(bind_function7, A, B, C, D, E, F, G);
    lua_wrapper_function_impl!(bind_function8, A, B, C, D, E, F, G, H);
    lua_wrapper_function_impl!(bind_function9, A, B, C, D, E, F, G, H, I);
    lua_wrapper_function_impl!(bind_function10, A, B, C, D, E, F, G, H, I, J);

    fn set_lua_path(&mut self, fieldname: &str, path: &str) {
        let mut buffer = String::new();
        let mut package = self.get::<LuaTable>(cstr!("package")).unwrap();
        let res = path.find(";;");
        if !res.is_none() {
            let dftmark = res.unwrap();
            if dftmark > 0 {
                // 添加前缀部分
                buffer.push_str(&path[..dftmark]);
                buffer.push_str(";");
            }
            // 添加默认路径
            let dft = package.get::<&str, String>(fieldname).unwrap_or_default();
            buffer.push_str(&dft);
            // 添加后缀部分
            if dftmark + 2 < path.len() {
                buffer.push_str(";");
                buffer.push_str(&path[dftmark + 2..]);
            }
        } else {
            buffer.push_str(path);
        }
        #[cfg(windows)] {
            let cur_path = env::current_dir().unwrap();
            buffer = buffer.replace("!", cur_path.to_str().unwrap());
        }
        package.set(fieldname, buffer);
    }
}
