#![allow(non_snake_case)]
#![allow(dead_code)]

use libc::c_int as int;
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