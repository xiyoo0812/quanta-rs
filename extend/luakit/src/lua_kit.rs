#![allow(non_snake_case)]
#![allow(dead_code)]

use std::env;
use std::cell::RefCell;

use lua::{ cstr, to_char, lua_State, lua_CFunction };

use crate::lua_stack::*;
use crate::lua_codec::*;
use crate::lua_function::*;
use crate::lua_buff::LuaBuf;
use crate::lua_base::LuaGuard;
use crate::lua_table::LuaTable;
use crate::lua_reference::Reference;

use crate::{ lua_set_function, lua_set_lua_function, call_table_warper, call_lua_warper, call_object_warper, call_function_inner };

pub struct Luakit {
    m_L: *mut lua_State
}

thread_local! {
    static LUA_BUFF: RefCell<LuaBuf> = RefCell::new(LuaBuf::new());
}

pub fn get_buff() -> &'static mut LuaBuf {
    LUA_BUFF.with(|lua_buff| {
        unsafe { &mut *lua_buff.as_ptr() }
    })
}

impl Luakit {
    pub fn new() -> Luakit {
        unsafe {
            let L =  lua::luaL_newstate();
            let mut kit = Luakit { m_L: L };
            lua::luaL_openlibs(L);
            lua::lua_checkstack(L, 1024);
            let mut lkit = kit.new_table(Some("luakit"));
            lkit.set_function("encode", encode);
            lkit.set_function("decode", decode);
            lkit.set_function("serialize", serialize);
            lkit.set_function("unserialize", unserialize);
            lkit.set_function0("luacodec", || Box::new(LuaCodec::new()));
            kit
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

    pub fn set<V>(&mut self, key: &str, value: V) where V: LuaPush {
        value.native_to_lua(self.m_L);
        unsafe { lua::lua_setglobal(self.m_L, to_char!(key)); }
    }

    pub fn get<R>(&mut self, key: &str) -> Option<R> where R: LuaRead {
        let _gl = LuaGuard::new(self.m_L);
        unsafe { lua::lua_getglobal(self.m_L, to_char!(key)); }
        return LuaRead::lua_to_native(self.m_L, -1);
    }

    pub fn get_path(&mut self, field: &str) -> String {
        let _gl = LuaGuard::new(self.m_L);
        unsafe {
            lua::lua_getglobal(self.m_L, cstr!("package"));
            lua::lua_getfield(self.m_L, -1, to_char!(field));
            lua::to_utf8(lua::lua_tostring(self.m_L, -1))
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
        let _gl = LuaGuard::new(self.m_L);
        unsafe {
            lua::lua_getglobal(self.m_L, cstr!("package"));
            lua::lua_getfield(self.m_L, -1, cstr!("searchers"));
            lua::lua_pushcfunction(self.m_L, f);
            lua::lua_rawseti(self.m_L, -2, 2);
        }
    }

    pub fn set_function(&mut self, name: &str, f : lua_CFunction) {
        let _gl = LuaGuard::new(self.m_L);
        unsafe {
            lua::lua_pushcfunction(self.m_L, f);
            lua::lua_setglobal(self.m_L, to_char!(name));
        }
    }

    pub fn new_table(&mut self, name: Option<&str>) -> LuaTable {
        unsafe {
            lua::lua_createtable(self.m_L, 0, 8);
            if !name.is_none() {
                lua::lua_pushvalue(self.m_L, -1);
                lua::lua_setglobal(self.m_L, to_char!(name.unwrap()));
            }
            LuaTable::new(self.m_L)
        }
    }

    pub fn run_file(&mut self, file: &String) ->Result<bool, String> {
        let _gl = LuaGuard::new(self.m_L);
        if lua::luaL_loadfile(self.m_L, file) != 0 {
            let err = lua::to_utf8(lua::lua_tostring(self.m_L, -1));
            println!("lua loadfile err: {}", err);
            return Err(err);
        }
        return lua_call_function(self.m_L, 0, 0);
    }

    pub fn run_script(&mut self, script: String) ->Result<bool, String> {
        let _gl = LuaGuard::new(self.m_L);
        unsafe {
            if lua::luaL_loadstring(self.m_L, to_char!(script)) != 0 {
                let err = lua::to_utf8(lua::lua_tostring(self.m_L, -1));
                println!("lua loadstring err: {}", err);
                return Err(err);
            }
        }
        return lua_call_function(self.m_L, 0, 0);
    }

    pub fn get_function(&mut self, function: &str) -> bool {
        return get_global_function(self.m_L, function);
    }

    pub fn call_function(&mut self) ->Result<bool, String> {
        return lua_call_function(self.m_L, 0, 0);
    }

    pub fn call(&mut self, func: &str) ->Result<bool, String> {
        return call_global_function(self.m_L, func, 0, 0);
    }

    pub fn table_call(&mut self, table: &str, func: &str) ->Result<bool, String> {
        return call_table_function(self.m_L, table, func, 0, 0);
    }

    fn set_lua_path(&mut self, fieldname: &str, path: &str) {
        let mut buffer = String::new();
        let mut package = self.get::<LuaTable>("package").unwrap();
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

    call_lua_warper!(call0, );
    call_lua_warper!(call1, A);
    call_lua_warper!(call2, A, B);
    call_lua_warper!(call3, A, B, C);
    call_lua_warper!(call4, A, B, C, D);
    call_lua_warper!(call5, A, B, C, D, E);
    call_lua_warper!(call6, A, B, C, D, E, F);
    call_lua_warper!(call7, A, B, C, D, E, F, G);
    call_lua_warper!(call8, A, B, C, D, E, F, G, H);
    call_lua_warper!(call9, A, B, C, D, E, F, G, H, I);
    call_lua_warper!(call10, A, B, C, D, E, F, G, H, I, J);
    
    lua_set_function!(set_function0,);
    lua_set_function!(set_function1, A);
    lua_set_function!(set_function2, A, B);
    lua_set_function!(set_function3, A, B, C);
    lua_set_function!(set_function4, A, B, C, D);
    lua_set_function!(set_function5, A, B, C, D, E);
    lua_set_function!(set_function6, A, B, C, D, E, F);
    lua_set_function!(set_function7, A, B, C, D, E, F, G);
    lua_set_function!(set_function8, A, B, C, D, E, F, G, H);
    lua_set_function!(set_function9, A, B, C, D, E, F, G, H, I);
    lua_set_function!(set_function10, A, B, C, D, E, F, G, H, I, J);

    lua_set_lua_function!(set_lfunction0,);
    lua_set_lua_function!(set_lfunction1, A);
    lua_set_lua_function!(set_lfunction2, A, B);
    lua_set_lua_function!(set_lfunction3, A, B, C);
    lua_set_lua_function!(set_lfunction4, A, B, C, D);
    lua_set_lua_function!(set_lfunction5, A, B, C, D, E);
    lua_set_lua_function!(set_lfunction6, A, B, C, D, E, F);
    lua_set_lua_function!(set_lfunction7, A, B, C, D, E, F, G);
    lua_set_lua_function!(set_lfunction8, A, B, C, D, E, F, G, H);
    lua_set_lua_function!(set_lfunction9, A, B, C, D, E, F, G, H, I);
    lua_set_lua_function!(set_lfunction10, A, B, C, D, E, F, G, H, I, J);

    call_table_warper!(table_call0, );
    call_table_warper!(table_call1, A);
    call_table_warper!(table_call2, A, B);
    call_table_warper!(table_call3, A, B, C);
    call_table_warper!(table_call4, A, B, C, D);
    call_table_warper!(table_call5, A, B, C, D, E);
    call_table_warper!(table_call6, A, B, C, D, E, F);
    call_table_warper!(table_call7, A, B, C, D, E, F, G);
    call_table_warper!(table_call8, A, B, C, D, E, F, G, H);
    call_table_warper!(table_call9, A, B, C, D, E, F, G, H, I);
    call_table_warper!(table_call10, A, B, C, D, E, F, G, H, I, J);
    
    call_object_warper!(object_call0, );
    call_object_warper!(object_call1, A);
    call_object_warper!(object_call2, A, B);
    call_object_warper!(object_call3, A, B, C);
    call_object_warper!(object_call4, A, B, C, D);
    call_object_warper!(object_call5, A, B, C, D, E);
    call_object_warper!(object_call6, A, B, C, D, E, F);
    call_object_warper!(object_call7, A, B, C, D, E, F, G);
    call_object_warper!(object_call8, A, B, C, D, E, F, G, H);
    call_object_warper!(object_call9, A, B, C, D, E, F, G, H, I);
    call_object_warper!(object_call10, A, B, C, D, E, F, G, H, I, J);

}
