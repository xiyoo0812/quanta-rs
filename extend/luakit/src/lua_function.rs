#![allow(non_snake_case)]
#![allow(unused_macros)]
#![allow(dead_code)]

use libc::c_int as int;
use libc::c_char as char;
use lua::{ cstr, lua_State };

//get global function
//-------------------------------------------------------------------------------
pub unsafe fn get_global_function(L: *mut lua_State, func: *const char) -> bool {
    lua::lua_getglobal(L, func);
    return lua::lua_isfunction(L, -1);
}

//call function
//-------------------------------------------------------------------------------
pub unsafe fn lua_call_function(L: *mut lua_State, arg_count: int, ret_count: int) -> Result<bool, String> {
    let func_idx = lua::lua_gettop(L) - arg_count;
    if func_idx <= 0 || !lua::lua_isfunction(L, func_idx) {
        return Err("function not found".to_string());
    }
    // 获取 Lua 调试信息（可选）
    lua::lua_getglobal(L, cstr!("debug"));
    lua::lua_getfield(L, -1, cstr!("traceback"));
    lua::lua_remove(L, -2);  // 移除 'debug'

    lua::lua_insert(L, func_idx);
    if lua::lua_pcall(L, arg_count, ret_count, func_idx) != 0 {
        let error_msg = lua::lua_tostring(L, -1).unwrap();
        lua::lua_pop(L, 2);  // 弹出错误信息和 traceback
        return Err(error_msg.to_string());
    }
    lua::lua_remove(L, -(ret_count + 1));  // 移除 'traceback'
    return Ok(true);
}

pub unsafe fn call_global_function(L: *mut lua_State, func: *const char, arg_count: int, ret_count: int) -> Result<bool, String> {
    if !get_global_function(L, func) {
        return Err("function not found".to_string());
    }
    return lua_call_function(L, arg_count, ret_count);
}

#[macro_export]
macro_rules! lua_call_function_impl {
    ($name:ident, $($p:ident),*) => {
        #[allow(unused_mut)]
        pub unsafe fn $name<$($p),*>(&mut self, func: *const char, retc: i32, $($p : $p, )*) -> Result<Vec<Reference>, String> where $($p : LuaPush),* {
            if !get_global_function(self.m_L, func) {
                return Err("function not found".to_string());
            }
            let mut argc = 0;
            $(
                argc += $p.native_to_lua(self.m_L);
            )*
            let res = lua_call_function(self.m_L, argc, retc);
            match res {
                Err(e) => return Err(e),
                Ok(_) => {
                    let mut ret = Vec::new();
                    for i in 0..retc {
                        ret.push(LuaRead::lua_to_native(self.m_L, -(retc - i)).unwrap());
                    }
                    lua::lua_pop(self.m_L, retc);
                    return Ok(ret);
                }
            }
        }
    };
}

#[macro_export]
macro_rules! variadic_return_impl {
    ($name:ident, $($p:ident),*) => {
        #[allow(unused_mut)]
        pub unsafe fn $name<$($p),*>(&mut self, $($p : $p, )*) -> i32 where $($p : LuaPush),* {
            let mut argc = 0;
            $(argc += $p.native_to_lua(self.m_L);)*
            return argc
        }
    }
}
