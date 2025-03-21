#![allow(non_snake_case)]
#![allow(dead_code)]

use std::fmt;
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};

use libc::c_int as int;
use libc::c_void as void;
use lua::{ ternary, to_cptr, lua_State, to_utf8, to_string };

use crate::lua_buff::LuaBuf;
use crate::lua_slice::Slice;
use crate::lua_base::LuaGuard;
use crate::LuaPush;

const TYPE_NIL: u8              = 0;
const TYPE_TRUE: u8             = 1;
const TYPE_FALSE: u8            = 2;
const TYPE_TAB_HEAD: u8         = 3;
const TYPE_TAB_TAIL: u8         = 4;
const TYPE_NUMBER: u8           = 5;
const TYPE_INT16: u8            = 6;
const TYPE_INT32: u8            = 7;
const TYPE_INT64: u8            = 8;
const TYPE_STRING8: u8          = 9;
const TYPE_STRING16: u8         = 10;
const TYPE_STRING32: u8         = 11;
const TYPE_ISTRING: u8          = 12;
const TYPE_STRINDEX: u8         = 13;
const TYPE_UNDEFINE: u8         = 14;
const TYPE_MAX: u8              = 15;

const MAX_ENCODE_DEPTH: u8      = 16;
const MAX_SHARE_STRING: usize   = 128;
const MAX_UINT8: u8             = u8::MAX - TYPE_MAX;
const UCHAR_MAX: usize          = u8::MAX as usize;
const USHRT_MAX: usize          = u16::MAX as usize;
const MAX_STRING_SIZE: usize    = 0xffffff;

thread_local! {
    static T_SSHARES: RefCell<Vec<String>> = RefCell::new(Vec::new());
}

// 错误类型定义
#[derive(Debug)]
pub enum CodecError {
    BufferOverflow,
    InvalidLength,
    DecodeError(String),
}

impl fmt::Display for CodecError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CodecError::BufferOverflow => write!(f, "Buffer overflow"),
            CodecError::InvalidLength => write!(f, "Invalid length"),
            CodecError::DecodeError(msg) => write!(f, "{}", msg),
        }
    }
}

fn serialize_value(buff: &mut LuaBuf, value: &[u8]) {
    buff.push_data(value);
}

fn serialize_quote(buff: &mut LuaBuf, str: &str, l: &str, r: &str) {
    serialize_value(buff, l.as_bytes());
    serialize_value(buff, str.as_bytes());
    serialize_value(buff, r.as_bytes());
}

fn serialize_udata(buff: &mut LuaBuf, data: *const void) {
    let udata = format!("userdata: {}", data as usize);
    serialize_quote(buff, &udata, "'", "'");
}

fn serialize_crcn(buff: &mut LuaBuf, count: u8, line: bool) {
    if line {
        serialize_value(buff, "\n".as_bytes());
        for _ in 1..count {
            serialize_value(buff, "    ".as_bytes());
        }
    }
}

fn serialize_string(L: *mut lua_State, buff: &mut LuaBuf, index: int) {
    serialize_value(buff, "'".as_bytes());
    let str = lua::lua_tostring(L, index);
    if !str.is_empty() {
        buff.push_data(str);
    }
    serialize_value(buff, "'".as_bytes());
}

fn serialize_table(L: *mut lua_State, buff: &mut LuaBuf, index: int, depth: u8, line: bool) {
    unsafe {
        let mut size = 0;
        let index = lua::lua_absindex(L, index);
        serialize_value(buff, "{".as_bytes());
        if crate::is_lua_array(L, index, false) {
            let rawlen = lua::lua_rawlen(L, index);
            for i in 1..=rawlen {
                if { let nfirst = size > 0; size += 1; nfirst } {
                    serialize_value(buff, ",".as_bytes());
                }
                serialize_crcn(buff, depth, line);
                lua::lua_rawgeti(L, index, i as i32);
                serialize_one(L, buff, -1, depth, line);
                lua::lua_pop(L, 1);
            }
        }
        else {
            if lua::lua_type(L, 3) == lua::LUA_TFUNCTION {
                let _gl = LuaGuard::new(L);
                lua::lua_pushvalue(L, 3);
                lua::lua_pushvalue(L, index);
                //执行sort方法，再序列化
                if lua::lua_pcall(L, 1, 1, 0) != 0 {
                    lua::lua_error(L);
                }
                let index = lua::lua_absindex(L, -1);
                lua::lua_pushnil(L);
                while lua::lua_next(L, index) != 0 {
                    if { let nfirst = size > 0; size += 1; nfirst } {
                        serialize_value(buff, ",".as_bytes());
                    }
                    lua::lua_geti(L, -1, 1);
                    lua::lua_geti(L, -2, 2);
                    serialize_crcn(buff, depth, line);
                    if lua::lua_type(L, -2) == lua::LUA_TNUMBER {
                        lua::lua_pushvalue(L, -2);
                        serialize_quote(buff, &to_utf8(lua::lua_tostring(L, -1)), "[", "]=");
                        lua::lua_pop(L, 1);
                    }
                    else if lua::lua_type(L, -2) == lua::LUA_TSTRING {
                        serialize_value(buff, lua::lua_tostring(L, -2));
                        serialize_value(buff, "=".as_bytes());
                    }
                    else {
                        serialize_one(L, buff, -2, depth, line);
                        serialize_value(buff, "=".as_bytes());
                    }
                    serialize_one(L, buff, -1, depth, line);
                    lua::lua_pop(L, 3);
                }
            }
            else {
                lua::lua_pushnil(L);
                while lua::lua_next(L, index) != 0 {
                    if { let nfirst = size > 0; size += 1; nfirst } {
                        serialize_value(buff, ",".as_bytes());
                    }
                    serialize_crcn(buff, depth, line);
                    if lua::lua_type(L, -2) == lua::LUA_TNUMBER {
                        lua::lua_pushvalue(L, -2);
                        serialize_quote(buff, &to_utf8(lua::lua_tostring(L, -1)), "[", "]=");
                        lua::lua_pop(L, 1);
                    }
                    else if lua::lua_type(L, -2) == lua::LUA_TSTRING {
                        serialize_value(buff, lua::lua_tostring(L, -2));
                        serialize_value(buff, "=".as_bytes());
                    }
                    else {
                        serialize_one(L, buff, -2, depth, line);
                        serialize_value(buff, "=".as_bytes());
                    }
                    serialize_one(L, buff, -1, depth, line);
                    lua::lua_pop(L, 1);
                }
            }
        }
        if size > 0 {
            serialize_crcn(buff, depth - 1, line);
        }
        serialize_value(buff, "}".as_bytes());
    }
}

pub fn serialize_one(L: *mut lua_State, buff: &mut LuaBuf, index: int, depth: u8, line: bool) {
    if depth > MAX_ENCODE_DEPTH {
        lua::luaL_error(L, "serialize can't pack too depth table");
    }
    unsafe {
        let ltype = lua::lua_type(L, index);
        match ltype {
            lua::LUA_TSTRING => serialize_string(L, buff, index),
            lua::LUA_TNIL => serialize_value(buff, "nil".as_bytes()),
            lua::LUA_TTABLE => serialize_table(L, buff, index, depth + 1, line),
            lua::LUA_TNUMBER => serialize_value(buff, lua::lua_tostring(L, index)),
            lua::LUA_TUSERDATA => serialize_udata(buff, lua::lua_touserdata(L, index)),
            lua::LUA_TLIGHTUSERDATA => serialize_udata(buff, lua::lua_touserdata(L, index)),
            lua::LUA_TBOOLEAN => serialize_value(buff, ternary!(lua::lua_toboolean(L, index), "true".as_bytes(), "false".as_bytes())),
            _ => serialize_quote(buff, &to_string(lua::lua_typename(L, ltype)), "'unsupport(", ")'"),
        }
    }
}
pub fn serialize(L: *mut lua_State) -> int {
    let buff = crate::get_buff();
    buff.clean();
    serialize_one(L, buff, 1, 1, lua::luaL_optinteger(L, 2, 0) > 0);
    unsafe { lua::lua_pushlstring_(L, to_cptr(buff.data()), buff.size()) };
    1
}

pub fn unserialize(L: *mut lua_State) -> int {
    let data = to_utf8(lua::lua_tolstring(L, 1));
    let script = format!("return {}", data);
    unsafe {
        if lua::luaL_loadbuffer(L, &script, script.len(), "unserialize") == 0 {
            if lua::lua_pcall(L, 0, 1, 0) == 0 {
                return 1;
            }
        }
        lua::lua_pushnil(L);
        lua::lua_insert(L, -2);
        return 2;
    }
}

//decode module
//-------------------------------------------------------------------------------
fn find_index(str: &str) -> i8 {
    T_SSHARES.with_borrow(|shares| {
        for (i, item) in shares.iter().enumerate() {
            if *item == str { return i as i8; }
        }
        -1
    })
}

fn find_string(index: usize) -> String {
    T_SSHARES.with(|shares| {
        shares.borrow().get(index).cloned().unwrap_or_else(|| "".to_string())
    })
}
fn value_encode<T: ?Sized>(buff: &mut LuaBuf, value: &T) {
    buff.write(value);
}

fn value_decode<T: Copy + Default>(slice: &mut Slice) -> Result<T, CodecError> {
    if let Some(value) = slice.read::<T>() {
        return Ok(value);
    }
    Err(CodecError::DecodeError("Failed to decode value".to_string()))
}

fn vu8_encode(buff: &mut LuaBuf, value: &[u8]) {
    buff.push_data(value);
}

fn string_write(buff: &mut LuaBuf, value: &[u8]) {
    let sz = value.len();
    if sz <= UCHAR_MAX {
        value_encode(buff, &TYPE_STRING8);
        value_encode(buff, &(sz as u8));
    } else if sz <= USHRT_MAX {
        value_encode(buff, &TYPE_STRING16);
        value_encode(buff, &(sz as u16));
    } else {
        value_encode(buff, &TYPE_STRING32);
        value_encode(buff, &(sz as u32));
    }
    if sz > 0 {
        vu8_encode(buff, value);
    }
}

fn integer_encode(buff: &mut LuaBuf, integer: i64) {
    if integer >= 0 && integer <= MAX_UINT8 as i64 {
        let ei = integer as u8 + TYPE_MAX;
        value_encode(buff, &ei);
        return;
    }
    if integer <= i16::MAX as i64 && integer >= i16::MIN as i64 {
        value_encode(buff, &TYPE_INT16);
        value_encode(buff, &(integer as i16));
        return;
    }
    if integer <= i32::MAX as i64 && integer >= i32::MIN as i64 {
        value_encode(buff, &TYPE_INT32);
        value_encode(buff, &(integer as i32));
        return;
    }
    value_encode(buff, &TYPE_INT64);
    value_encode(buff, &integer);
}

fn number_encode(buff: &mut LuaBuf, number: f64) {
    value_encode(buff, &TYPE_NUMBER);
    value_encode(buff, &number);
}

fn string_encode(L: *mut lua_State, buff: &mut LuaBuf, index: int) {
    let ptr = lua::lua_tolstring(L, index);
    if ptr.len() >= MAX_STRING_SIZE {
        lua::luaL_error(L, &format!("encode string can't pack too long {} string", ptr.len()));
    }
    string_write(buff, ptr);
}

fn index_encode(L: *mut lua_State, buff: &mut LuaBuf, index: int) {
    let ptr = lua::lua_tolstring(L, index);
    if ptr.len() >= MAX_STRING_SIZE {
        lua::luaL_error(L, &format!("encode string can't pack too long {} string", ptr.len()));
    }
    let sz = ptr.len();
    if sz > UCHAR_MAX || sz == 0 {
        string_write(buff, ptr);
        return;
    }
    let str = unsafe { std::str::from_utf8_unchecked(ptr) };
    let sindex = find_index(str);
    if sindex < 0 {
        if T_SSHARES.take().len() < MAX_SHARE_STRING {
            value_encode(buff, &TYPE_ISTRING);
        } else {
            value_encode(buff, &TYPE_STRING8);
        }
        value_encode(buff, &(sz as u8));
        vu8_encode(buff, ptr);
        return;
    }
    value_encode(buff, &TYPE_STRINDEX);
    value_encode(buff, &(sindex as u8));
}

fn encode_one(L: *mut lua_State, buff: &mut LuaBuf, idx: int, depth: u8, isindex: bool) {
    if depth > MAX_ENCODE_DEPTH {
        lua::luaL_error(L, "encode can't pack too depth table");
    }
    unsafe {
        let ltype = lua::lua_type(L, idx);
        match ltype {
            lua::LUA_TNIL => value_encode(buff, &TYPE_NIL),
            lua::LUA_TTABLE => table_encode(L, buff, idx, depth + 1),
            lua::LUA_TSTRING => ternary!(isindex, index_encode(L, buff, idx), string_encode(L, buff, idx)),
            lua::LUA_TBOOLEAN => ternary!(lua::lua_toboolean(L, idx) , value_encode(buff, &TYPE_TRUE), value_encode(buff, &TYPE_FALSE)),
            lua::LUA_TNUMBER => {
                let is_int = lua::lua_isinteger(L, idx);
                ternary!(is_int, integer_encode(buff, lua::lua_tointeger(L, idx) as i64), number_encode(buff, lua::lua_tonumber(L, idx)));
            },
            _ => value_encode(buff, &TYPE_UNDEFINE),
        }
    }
}

fn table_encode(L: *mut lua_State, buff: &mut LuaBuf, index: int, depth: u8) {
    unsafe {
        let index = lua::lua_absindex(L, index);
        value_encode(buff, &TYPE_TAB_HEAD);
        lua::lua_pushnil(L);
        while lua::lua_next(L, index) != 0 {
            encode_one(L, buff, -2, depth, true);
            encode_one(L, buff, -1, depth, false);
            lua::lua_pop(L, 1);
        }
        value_encode(buff, &TYPE_TAB_TAIL);
    }
}

fn encode_slice(L: *mut lua_State, index: int, num: i32) -> Slice<'static> {
    if num > i8::MAX as i32 || num < 0 {
        lua::luaL_error(L, "encode can't pack too many args");
    }
    T_SSHARES.take().clear();
    let buff = crate::get_buff();
    buff.clean();
    buff.write(&(num as u8));
    for i in 0..num {
        encode_one(L, buff, index + i, 0, false);
    }
    buff.get_slice(None, None)
}

pub fn encode(L: *mut lua_State) ->int {
    let slice = encode_slice(L, 1, 1);
    slice.string(L)
}

fn string_decode(L: *mut lua_State, sz: usize, slice: &mut Slice, isindex: bool) -> Result<(), CodecError> {
    if sz == 0 {
        lua::lua_pushstring(L, "");
        return Ok(());
    }
    if let Some(val) = slice.peek(sz, 0) {
        val.native_to_lua(L);
        if isindex {
            T_SSHARES.with_borrow_mut(|shares| {
                unsafe { shares.push(String::from_utf8_unchecked(val.to_vec())); }
            });
        }
        slice.erase(sz);
        return Ok(());
    }
    Err(CodecError::DecodeError("decode string is out of range".to_string()))
}

fn index_decode(L: *mut lua_State, slice: &mut Slice) -> Result<(), CodecError> {
    let index = value_decode::<u8>(slice)?;
    let val = find_string(index as usize);
    lua::lua_pushlstring(L, &val);
    Ok(())
}

fn decode_value(L: *mut lua_State, slice: &mut Slice, dtype: u8) -> Result<(), CodecError> {
    unsafe {
        match dtype {
            TYPE_NIL => lua::lua_pushnil(L),
            TYPE_TRUE => lua::lua_pushboolean(L, 1),
            TYPE_FALSE => lua::lua_pushboolean(L, 0),
            TYPE_NUMBER=> lua::lua_pushinteger(L, value_decode::<f64>(slice)? as isize),
            TYPE_INT64 => lua::lua_pushinteger(L, value_decode::<i64>(slice)? as isize),
            TYPE_INT32 => lua::lua_pushinteger(L, value_decode::<i32>(slice)? as isize),
            TYPE_INT16 => lua::lua_pushinteger(L, value_decode::<i16>(slice)? as isize),
            TYPE_STRING8 => string_decode(L, value_decode::<u8>(slice)? as usize, slice, false)?,
            TYPE_STRING16 => string_decode(L, value_decode::<u16>(slice)? as usize, slice, false)?,
            TYPE_STRING32 => string_decode(L, value_decode::<u32>(slice)? as usize, slice, false)?,
            TYPE_ISTRING => string_decode(L, value_decode::<u8>(slice)? as usize, slice, true)?,
            TYPE_STRINDEX => index_decode(L, slice)?,
            TYPE_TAB_HEAD => table_decode(L, slice)?,
            TYPE_TAB_TAIL => {},
            TYPE_UNDEFINE => lua::lua_pushstring(L, "undefine"),
            _ => lua::lua_pushinteger(L, (dtype - TYPE_MAX) as isize),
        }
        Ok(())
    }
}

fn decode_one(L: *mut lua_State, slice: &mut Slice) -> Result<u8, CodecError> {
    let dtype = value_decode::<u8>(slice)?;
    decode_value(L, slice, dtype)?;
    Ok(dtype)
}

fn table_decode(L: *mut lua_State, slice: &mut Slice)-> Result<(), CodecError> {
    unsafe {
        lua::lua_createtable(L, 0, 8);
        loop {
            let dtype = decode_one(L, slice)?;
            if dtype == TYPE_TAB_TAIL {
                break;
            }
            decode_one(L, slice)?;
            lua::lua_rawset(L, -3);
        }
        Ok(())
    }
}

pub fn decode_slice(L: *mut lua_State, slice: &mut Slice) -> Result<i32, CodecError> {
    T_SSHARES.take().clear();
    let top = unsafe { lua::lua_gettop(L)};
    let argnum = value_decode::<u8>(slice)? as i32;
    unsafe { lua::lua_checkstack(L, argnum)};
    loop {
        if let Some(dtype) = slice.read::<u8>() {
            decode_value(L, slice, dtype)?;
            continue;
        }
        break;
    }
    let getnum = unsafe { lua::lua_gettop(L) - top };
    if argnum != getnum {
        let msg = format!("decode arg num expect {}, but get {}", argnum, getnum);
        return Err(CodecError::DecodeError(msg));
    }
    Ok(getnum)
}

pub fn decode(L: *mut lua_State) -> int {
    let buff = crate::get_buff();
    buff.clean();
    buff.push_data(lua::lua_tolstring(L, 1));
    let mut slice = buff.get_slice(None, None);
    let res = decode_slice(L, &mut slice);
    match res {
        Ok(argc) => argc,
        Err(e) => {
            lua::luaL_error(L, &e.to_string());
        }
    }
}

// 核心编解码trait
pub trait Codec {
    fn encode(&mut self, L: *mut lua_State, index: i32) -> Vec<u8> {
        let n = unsafe { lua::lua_gettop(L) } - index + 1;
        let slice = encode_slice(L, index, n);
        slice.contents().to_vec()
    }
    fn decode(&mut self, L: *mut lua_State) -> Result<i32, CodecError>;
}

// 基础编解码器实现
pub struct BaseCodec {
    error: String,
    packet_len: i32,
}

impl BaseCodec {
    pub fn new() -> Self {
        Self {
            packet_len: 0,
            error: "".to_string(),
        }
    }

    pub fn load_packet(&self, slice: &Slice) -> i32 {
        let data_len = slice.size() as i32;
        if data_len == 0 { return 0; }
        if let Some(packet_len) = slice.touch::<i32>() {
            if packet_len > 0xffffff { return -1; }
            if packet_len > data_len { return 0; }
            if !slice.peek(packet_len as usize, 0).is_some() {
                return 0;
            }
            return packet_len + 4;
        }
        0
    }

    pub fn error(&mut self, err: String) {
        self.error = err;
    }

    pub fn failed(&self) -> bool {
        !self.error.is_empty()
    }

    pub fn err(&self) -> &str {
        &self.error
    }

    pub fn decode_impl(&mut self, L: *mut lua_State, slice: &mut Slice) -> Result<i32, CodecError>{
        decode_slice(L, slice)
    }
}


// Lua专用编解码器
pub struct LuaCodec {
    base: BaseCodec,
}

impl LuaCodec {
    pub fn new() -> Self {
        Self { base: BaseCodec::new() }
    }

    pub fn decode_data(&mut self, L: *mut lua_State, data: &[u8]) -> Result<i32, CodecError> {
        let buff = crate::get_buff();
        buff.clean();
        buff.push_data(data);
        self.decode(L)
    }
}

impl Deref for LuaCodec {
    type Target = BaseCodec;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for LuaCodec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl Codec for LuaCodec {
    fn decode(&mut self, L: *mut lua_State) -> Result<i32, CodecError>{
        let buff = crate::get_buff();
        let mut slice = buff.get_slice(None, Some(4));
        self.base.decode_impl(L, &mut slice)
    }
}