#![allow(non_snake_case)]
#![allow(dead_code)]

extern crate serde_yml;

use lua::lua_State;
use libc::c_int as int;

use serde_yml::{ Mapping, Number, Value };
use std::{fs::File, fs::OpenOptions, io::Read, io::Write};

pub const MAX_ENCODE_DEPTH: u32     = 16;

fn encode_number(L: *mut lua_State, idx: i32) -> Value {
    if lua::lua_isinteger(L, idx) {
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
        lua::luaL_error(L, "encode can't pack too depth table");
    }
    let ttype = lua::lua_type(L, idx);
    match ttype {
        lua::LUA_TNUMBER => return encode_number(L, idx),
        lua::LUA_TTABLE => return encode_table(L, emy_as_arr, idx, depth + 1),
        lua::LUA_TBOOLEAN => {
            let val = lua::lua_toboolean(L, idx);
            return Value::Bool(val);
        }
        lua::LUA_TSTRING=> {
            let val = lua::to_utf8(lua::lua_tostring(L, idx));
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

pub unsafe fn decode_array(L: *mut lua_State, val: &Vec<Value>) {
    lua::lua_createtable(L, val.len() as i32, 0);
    for (i, v) in val.iter().enumerate() {
        lua::lua_pushinteger(L, i as isize + 1);
        decode_one(L, v);
        lua::lua_settable(L, -3);
    }
}

pub unsafe fn decode_object(L: *mut lua_State, val: &Mapping) {
    lua::lua_createtable(L, 0, val.len() as i32);
    for (k, v) in val.iter() {
        decode_one(L, k);
        decode_one(L, v);
        lua::lua_settable(L, -3);
    }
}

pub unsafe fn decode_one(L: *mut lua_State, value: &Value) -> int {
    match value {
        Value::Tagged(_) => {},
        Value::Null => lua::lua_pushnil(L),
        Value::Number(val) => decode_number(L, val),
        Value::Bool(val) => lua::lua_pushboolean(L, *val as i32),
        Value::Sequence(val) => decode_array(L, val),
        Value::Mapping(val) => decode_object(L, val),
        Value::String(val) => lua::lua_pushlstring(L, val),
    }
    return 1;
}

pub fn decode_core(L: *mut lua_State, yaml: String) -> int {
    let res = serde_yml::from_str::<Value>(yaml.as_str());
    match res {
        Ok(mut val) => {
            if val.apply_merge().is_err() {
                lua::luaL_error(L, "encode can't unpack json");
            }
            unsafe { decode_one(L, &val) }
        },
        Err(_) => lua::luaL_error(L, "encode can't unpack json")
    }
}

pub fn decode(L: *mut lua_State) -> int {    
    let yaml = lua::to_utf8(lua::lua_tolstring(L, 1));
    return decode_core(L, yaml);
}

pub fn encode(L: *mut lua_State) -> int {
    unsafe {
        let emy_as_arr = lua::lua_toboolean(L, 2);
        let val = encode_one(L, emy_as_arr, 1, 0);
        let x: Result<_, _> = serde_yml::to_string(&val);
        match x {
            Ok(x) => lua::lua_pushlstring(L, &x),
            Err(_) => lua::luaL_error(L, "encode can't pack too depth table"),
        }
        return 1;
    }
}

pub fn open(L: *mut lua_State) -> int {
    let filename = lua::to_utf8(lua::lua_tolstring(L, 1));
    let res = File::open(filename);
    match res {
        Ok(mut f) => {
            let mut yaml = String::new();
            match f.read_to_string(&mut yaml) {
                Ok(_) => { return decode_core(L, yaml); },
                Err(e) => lua::luaL_error(L, &e.to_string())
            };
        },
        Err(e) => lua::luaL_error(L, &e.to_string())
    }
}

pub fn save(L: *mut lua_State) -> int {
    unsafe {
        let filename = lua::to_utf8(lua::lua_tolstring(L, 1));
        let res = OpenOptions::new().write(true).create(true).open(filename);
        match res {
            Ok(mut f) => {
                let val = encode_one(L, true, 2, 0);
                let eres  = serde_yml::to_string(&val);
                match eres {
                    Ok(x) => {
                        match f.write_all(x.as_bytes()) {
                            Ok(_) => return 0,
                            Err(e) => lua::luaL_error(L, &e.to_string())
                        };
                    },
                    Err(e) => lua::luaL_error(L, &e.to_string())
                }
            },
            Err(e) => lua::luaL_error(L, &e.to_string())
        }
    }
}

