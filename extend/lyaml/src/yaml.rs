#![allow(non_snake_case)]
#![allow(dead_code)]

extern crate serde_yml;

use libc::c_int as int;
use lua::{ cstr,  ternary, lua_State };

use serde_yml::{ Mapping, Number, Value };

pub const MAX_ENCODE_DEPTH: u32     = 16;

fn encode_number(L: *mut lua_State, idx: i32) -> Value {
    if unsafe { lua::lua_isinteger(L, idx) } == 1 {
        let val = lua::lua_tointeger(L, idx);
        return Value::Number(Number::from(val));
    }
    let val = lua::lua_tonumber(L, idx);
    return Value::Number(Number::from(val));
}

unsafe fn encode_array(L: *mut lua_State, emy_as_arr: bool, index: i32, depth: u32) -> Value {
    let mut values: Vec<Value> = vec![];
    let raw_len = lua::lua_rawlen(L, index) as i32;
    for i in 1..=raw_len {
        lua::lua_rawgeti(L, index, i);
        values.push(encode_one(L, emy_as_arr, -1, depth));
        lua::lua_pop(L, 1);
    }
    return Value::Sequence(values);
}

unsafe fn encode_table(L: *mut lua_State, emy_as_arr: bool, idx: i32, depth: u32) -> Value {
    let index = lua::lua_absindex(L, idx);
    if !luakit::is_lua_array(L, index, emy_as_arr) {
        lua::lua_pushnil(L);
        let mut valmap = Mapping::new();
        while lua::lua_next(L, index) != 0 {
            let key = encode_one(L, emy_as_arr, -2, depth);
            let val = encode_one(L, emy_as_arr, -1, depth);
            valmap.insert(key, val);
            lua::lua_pop(L, 1);
        }
        return Value::Mapping(valmap);
    }
    return encode_array(L, emy_as_arr, index, depth);
}

pub unsafe fn encode_one(L: *mut lua_State, emy_as_arr: bool, idx: i32, depth: u32) -> Value {
    if depth > MAX_ENCODE_DEPTH {
        lua::luaL_error(L, cstr!("encode can't pack too depth table"));
    }
    let ttype = lua::lua_type(L, idx);
    match ttype {
        lua::LUA_TNUMBER => return encode_number(L, idx),
        lua::LUA_TTABLE => return encode_table(L, emy_as_arr, idx, depth + 1),
        lua::LUA_TBOOLEAN => {
            let val = ternary!(lua::lua_toboolean(L, idx), true, false);
            return Value::Bool(val);
        }
        lua::LUA_TSTRING=> {
            let val = lua::lua_tostring(L, idx).unwrap_or_default();
            return Value::String(val);
        }
        lua::LUA_TNIL => return Value::String("unsupported nil".to_string()),
        lua::LUA_TNONE => return Value::String("unsupported none".to_string()),
        lua::LUA_TTHREAD => return Value::String("unsupported thread".to_string()),
        lua::LUA_TFUNCTION => return Value::String("unsupported function".to_string()),
        lua::LUA_TUSERDATA => return Value::String("unsupported userdata".to_string()),
        lua::LUA_TLIGHTUSERDATA => return Value::String("unsupported luserdata".to_string()),
        _ => return Value::String("unsupported datatype".to_string())
    }
}

pub unsafe fn decode_number(L: *mut lua_State, val: &Number) {
    if val.is_i64() {
        lua::lua_pushinteger(L, val.as_i64().unwrap() as isize);
        return
    }
    lua::lua_pushnumber(L, val.as_f64().unwrap());
}

pub unsafe fn decode_array(L: *mut lua_State, val: &Vec<Value>, numkeyable: bool) {
    lua::lua_createtable(L, val.len() as i32, 0);
    for (i, v) in val.iter().enumerate() {
        lua::lua_pushinteger(L, i as isize + 1);
        decode_one(L, v, numkeyable);
        lua::lua_settable(L, -3);
    }
}

pub unsafe fn decode_object(L: *mut lua_State, val: &Mapping, numkeyable: bool) {
    lua::lua_createtable(L, 0, val.len() as i32);
    for (k, v) in val.iter() {
        decode_one(L, k, numkeyable);
        decode_one(L, v, numkeyable);
        lua::lua_settable(L, -3);
    }
}

pub unsafe fn decode_one(L: *mut lua_State, value: &Value, numkeyable: bool) -> int {
    match value {
        Value::Tagged(_) => {},
        Value::Null => lua::lua_pushnil(L),
        Value::Number(val) => decode_number(L, val),
        Value::Bool(val) => lua::lua_pushboolean(L, *val as i32),
        Value::Sequence(val) => decode_array(L, val, numkeyable),
        Value::Mapping(val) => decode_object(L, val, numkeyable),
        Value::String(val) => lua::lua_pushlstring(L, val.as_ptr() as *const i8, val.len() as usize),
    }
    return 1;
}

pub fn decode_core(L: *mut lua_State, numkeyable: bool, yaml: String) -> int {
    let res = serde_yml::from_str::<Value>(yaml.as_str());
    match res {
        Ok(mut val) => {
            if val.apply_merge().is_err() {
                lua::luaL_error(L, cstr!("encode can't unpack json"));
            }
            unsafe { decode_one(L, &val, numkeyable) }
        },
        Err(_) => lua::luaL_error(L, cstr!("encode can't unpack json"))
    }
}

pub fn decode(L: *mut lua_State) -> int {
    let yaml = lua::lua_tolstring(L, 1).unwrap_or_default();
    let numkeyable = lua::lua_toboolean(L, 2);
    return decode_core(L, numkeyable, yaml);
}

pub fn encode(L: *mut lua_State) -> int {
    unsafe {
        let emy_as_arr = lua::lua_toboolean(L, 2);
        let val = encode_one(L, emy_as_arr, 1, 0);
        let x: Result<_, _> = serde_yml::to_string(&val);
        match x {
            Ok(x) => lua::lua_pushlstring(L, x.as_ptr() as *const i8, x.len() as usize),
            Err(_) => lua::luaL_error(L, cstr!("encode can't pack too depth table")),
        }
        return 1;
    }
}