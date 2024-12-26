#![allow(non_snake_case)]
#![allow(dead_code)]

extern crate lua;
extern crate serde;
extern crate serde_json;

use lua::lua_State;
use std::ffi::CStr;
use libc::c_int as int;
use libc::size_t as size_t;
use serde_json::Value as JsonValue;

use std::str;

pub const MAX_ENCODE_DEPTH: u32     = 16;

pub unsafe fn encode(L: *mut lua_State) -> int {
    return encode_impl(L);
}

pub unsafe fn encode_impl(L: *mut lua_State) -> int {
    let emy_as_arr = lua::lua_toboolean(L, 2) != 0;
    let val = encode_one(L, emy_as_arr, 1, 0);
    let x = serde_json::to_string(&val);
    match x {
        Ok(x) => lua::lua_pushstring(L, x.as_ptr() as *const i8),
        Err(_) => lua::luaL_error(L, cstr!("encode can't pack too depth table")),
    }
    return 1;
}

unsafe fn encode_number(L: *mut lua_State, idx: i32) -> JsonValue {
    if lua::lua_isnumber(L, idx) == 1 {
        let val = lua::lua_tonumber(L, idx);
        return serde_json::json!(val);
    }
    let val = lua::lua_tointeger(L, idx);
    return serde_json::json!(val);
}

unsafe fn load_lua_string(L: *mut lua_State, idx: i32) -> String {
    let mut len : size_t = 0;
    let val = lua::lua_tolstring(L, idx, &mut len);
    let res = CStr::from_ptr(val).to_str();
    match res {
        Ok(x) => return x.to_string(),
        Err(_) => lua::luaL_error(L, cstr!("encode can't pack key"))
    }
    return "".to_string();
}

unsafe fn encode_key(L: *mut lua_State, idx: i32) -> String {
    let ttype = lua::lua_type(L, idx);
    match ttype {
        lua::LUA_TSTRING=> return load_lua_string(L, idx),
        lua::LUA_TNUMBER=> {
            let val = lua::lua_tonumber(L, idx);
            return val.to_string();
        },
        _ => lua::luaL_error(L, cstr!("encode can't pack key"))
    }
    return "".to_string();
}

unsafe fn encode_table(L: *mut lua_State, emy_as_arr: bool, idx: i32, depth: u32) -> JsonValue {
    lua::lua_pushnil(L);
    let mut doc: JsonValue = serde_json::json!({});
    while lua::lua_next(L, idx) != 0 {
        let key = encode_key(L, -2);
        let val = encode_one(L, emy_as_arr, -1, depth);
        doc[key] = val;
    }
    return doc;
}

pub unsafe fn encode_one(L: *mut lua_State, emy_as_arr: bool, idx: i32, depth: u32) -> JsonValue {
    if depth > MAX_ENCODE_DEPTH {
        lua::luaL_error(L, cstr!("encode can't pack too depth table"));
    }
    let ttype = lua::lua_type(L, idx);
    match ttype {
        lua::LUA_TNIL => return serde_json::json!(null),
        lua::LUA_TNUMBER => return encode_number(L, idx),
        lua::LUA_TTABLE => return encode_table(L, emy_as_arr, idx, depth + 1),
        lua::LUA_TBOOLEAN => {
            let val = ternary!(lua::lua_toboolean(L, idx) != 0, "true", "false");
            return serde_json::json!(val);
        }
        lua::LUA_TSTRING=> {
            let val = lua::lua_tonumber(L, idx);
            return serde_json::json!(val);
        }
        lua::LUA_TTHREAD => return serde_json::json!("unsupported thread"),
        lua::LUA_TFUNCTION => return serde_json::json!("unsupported function"),
        lua::LUA_TUSERDATA => return serde_json::json!("unsupported userdata"),
        lua::LUA_TLIGHTUSERDATA => return serde_json::json!("unsupported luserdata"),
        _ => return serde_json::json!("unsupported datatype")
    }
}
