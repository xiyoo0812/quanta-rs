#![allow(non_snake_case)]
#![allow(dead_code)]

extern crate toml;

use libc::c_int as int;
use toml::{ Value, Table };
use lua::{ cstr, to_char, ternary, lua_State };
use std::{fs::File, fs::OpenOptions, io::Read, io::Write};

pub const MAX_ENCODE_DEPTH: u32     = 16;

fn encode_number(L: *mut lua_State, idx: i32) -> Value {
    if unsafe { lua::lua_isinteger(L, idx) } == 1 {
        let val = lua::lua_tointeger(L, idx);
        return Value::Integer(val as i64);
    }
    let val = lua::lua_tonumber(L, idx);
    return Value::Float(val);
}

fn encode_key(L: *mut lua_State, idx: i32) -> String {
    let ttype = unsafe { lua::lua_type(L, idx) };
    match ttype {
        lua::LUA_TSTRING=> {
            let val: Option<String> = lua::lua_tolstring(L, idx);
            return val.unwrap_or_default();
        },
        lua::LUA_TNUMBER=> {
            let val = lua::lua_tonumber(L, idx);
            return val.to_string();
        },
        _ => lua::luaL_error(L, cstr!("encode can't pack key"))
    }
}

unsafe fn encode_array(L: *mut lua_State, emy_as_arr: bool, index: i32, depth: u32) -> Value {
    let mut values: Vec<Value> = vec![];
    let raw_len = lua::lua_rawlen(L, index) as i32;
    for i in 1..=raw_len {
        lua::lua_rawgeti(L, index, i);
        values.push(encode_one(L, emy_as_arr, -1, depth));
        lua::lua_pop(L, 1);
    }
    return Value::Array(values);
}

unsafe fn encode_table(L: *mut lua_State, emy_as_arr: bool, idx: i32, depth: u32) -> Value {
    let index = lua::lua_absindex(L, idx);
    if !luakit::is_lua_array(L, index, emy_as_arr) {
        lua::lua_pushnil(L);
        let mut valmap = Table::new();
        while lua::lua_next(L, index) != 0 {
            let key = encode_key(L, -2);
            let val = encode_one(L, emy_as_arr, -1, depth);
            valmap.insert(key, val);
            lua::lua_pop(L, 1);
        }
        return Value::Table(valmap);
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
            return Value::Boolean(val);
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

pub unsafe fn decode_array(L: *mut lua_State, val: &Vec<Value>, numkeyable: bool) {
    lua::lua_createtable(L, val.len() as i32, 0);
    for (i, v) in val.iter().enumerate() {
        lua::lua_pushinteger(L, i as isize + 1);
        decode_one(L, v, numkeyable);
        lua::lua_settable(L, -3);
    }
}

pub unsafe fn decode_key(L: *mut lua_State, val: &String, numkeyable: bool) {
    if numkeyable {
        let res = val.parse::<isize>();
        match res {
            Ok(val) => lua::lua_pushinteger(L, val),
            Err(_) => lua::lua_pushlstring(L, to_char!(val), val.len() as usize)
        }
        return
    }
    lua::lua_pushlstring(L, to_char!(val), val.len() as usize);
}

pub unsafe fn decode_object(L: *mut lua_State, val: &Table, numkeyable: bool) {
    lua::lua_createtable(L, 0, val.len() as i32);
    for (k, v) in val.iter() {
        decode_key(L, k, numkeyable);
        decode_one(L, v, numkeyable);
        lua::lua_settable(L, -3);
    }
}

pub unsafe fn decode_one(L: *mut lua_State, value: &Value, numkeyable: bool) -> int {
    match value {
        Value::Float(val) => lua::lua_pushnumber(L, *val as f64),
        Value::Boolean(val) => lua::lua_pushboolean(L, *val as i32),
        Value::Integer(val) => lua::lua_pushinteger(L, *val as isize),
        Value::Array(val) => decode_array(L, val, numkeyable),
        Value::Table(val) => decode_object(L, val, numkeyable),
        Value::Datetime(val) => lua::lua_pushstring(L, to_char!(val.to_string())),
        Value::String(val) => lua::lua_pushlstring(L, to_char!(val), val.len() as usize),
    }
    return 1;
}

pub fn decode_core(L: *mut lua_State, numkeyable: bool, toml: String) -> int {
    let res = toml::from_str::<Value>(toml.as_str());
    match res {
        Ok(val) => unsafe { decode_one(L, &val, numkeyable) },
        Err(_) => lua::luaL_error(L, cstr!("encode can't unpack json"))
    }
}

pub fn decode(L: *mut lua_State) -> int {
    let toml = lua::lua_tolstring(L, 1).unwrap_or_default();
    let numkeyable = lua::lua_toboolean(L, 2);
    return decode_core(L, numkeyable, toml);
}

pub fn encode(L: *mut lua_State) -> int {
    unsafe {
        let emy_as_arr = lua::lua_toboolean(L, 2);
        let val = encode_one(L, emy_as_arr, 1, 0);
        let x = toml::to_string(&val);
        match x {
            Ok(x) => lua::lua_pushlstring(L, to_char!(x), x.len() as usize),
            Err(_) => lua::luaL_error(L, cstr!("encode can't pack too depth table")),
        }
        return 1;
    }
}

pub fn open(L: *mut lua_State) -> int {
    let filename = lua::lua_tolstring(L, 1).unwrap_or_default();
    let res = File::open(filename);
    match res {
        Ok(mut f) => {
            let mut toml = String::new();
            match f.read_to_string(&mut toml) {
                Ok(_) => { return decode_core(L, true, toml); },
                Err(e) => lua::luaL_error(L, to_char!(e.to_string()))
            };
        },
        Err(e) => lua::luaL_error(L, to_char!(e.to_string()))
    }
}

pub fn save(L: *mut lua_State) -> int {
    unsafe {
        let filename = lua::lua_tolstring(L, 1).unwrap_or_default();
        let res = OpenOptions::new().write(true).create(true).open(filename);
        match res {
            Ok(mut f) => {
                let val = encode_one(L, true, 2, 0);
                let eres  = toml::to_string(&val);
                match eres {
                    Ok(x) => {
                        match f.write_all(x.as_bytes()) {
                            Ok(_) => return 0,
                            Err(e) => lua::luaL_error(L, to_char!(e.to_string()))
                        };
                    },
                    Err(e) => lua::luaL_error(L, to_char!(e.to_string()))
                }
            },
            Err(e) => lua::luaL_error(L, to_char!(e.to_string()))
        }
    }
}
