#![allow(non_snake_case)]
#![allow(dead_code)]
#[allow(unused_macros)]

use libc::c_int as int;
use libc::c_char as char;
use lua::{ cstr, lua_State };

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

pub unsafe fn get_global_function(L: *mut lua_State, func: *const char) -> bool {
    lua::lua_getglobal(L, func);
    return lua::lua_isfunction(L, -1);
}

pub unsafe fn call_global_function(L: *mut lua_State, func: *const char, arg_count: int, ret_count: int) -> Result<bool, String> {
    if !get_global_function(L, func) {
        return Err("function not found".to_string());
    }
    return lua_call_function(L, arg_count, ret_count);
}

#[macro_export]
macro_rules! lua_call_function {
    ($L:expr, $f:expr) => {
        luakit::lua_call_function(L, 0, 0);
        return ();
    };
    ($L:expr, $f:expr, $ret_count:expr) => {
        luakit::lua_call_function(L, 0, ret_count);
        return luakit::lua_to_native_mutil(L, ret_count);
    };
    // 匹配单个表达式
    ($L:expr, $f:expr, $ret_count:expr, $e:expr) => {
        native_to_lua($L, $e);
        luakit::lua_call_function(L, 0, 0)
        return luakit::lua_to_native_mutil(L, ret_count);
    };
    // 匹配两个或更多表达式
    ($L:expr, $f:expr, $ret_count:expr, $e:expr, $($es:expr),+) => {{
        native_to_lua($L, $e);
        return lua_call_function!(L, ret_count, $($es),+);
    }};
}
