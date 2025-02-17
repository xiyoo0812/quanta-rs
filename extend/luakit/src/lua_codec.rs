#![allow(non_snake_case)]
#![allow(dead_code)]

use std::ptr;

use libc::c_int as int;
use libc::c_void as void;
use lua::{ cstr, ternary, to_char, to_cptr, lua_State, to_utf8, to_string };

use crate::lua_buff::LuaBuf;
use crate::lua_slice::Slice;
use crate::lua_base::LuaGuard;

const TYPE_NIL: u8          = 0;
const TYPE_TRUE: u8         = 1;
const TYPE_FALSE: u8        = 2;
const TYPE_TAB_HEAD: u8     = 3;
const TYPE_TAB_TAIL: u8     = 4;
const TYPE_NUMBER: u8       = 5;
const TYPE_INT16: u8        = 6;
const TYPE_INT32: u8        = 7;
const TYPE_INT64: u8        = 8;
const TYPE_STRING8: u8      = 9;
const TYPE_STRING16: u8     = 10;
const TYPE_STRING32: u8     = 11;
const TYPE_ISTRING: u8      = 12;
const TYPE_STRINDEX: u8     = 13;
const TYPE_UNDEFINE: u8     = 14;
const TYPE_MAX: u8          = 15;

const MAX_ENCODE_DEPTH: u8  = 16;
const MAX_SHARE_STRING: u8  = 128;
const MAX_UINT8: u8         = 255 - TYPE_MAX;
const MAX_STRING_SIZE: u32  = 0xffffff;

// 错误类型定义
#[derive(Debug)]
pub enum CodecError {
    BufferOverflow,
    InvalidLength,
    DecodeError(String),
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
        for _ in 1..=count {
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
                if lua::lua_pcall(L, 1, 1, -2) == 0 {
                    lua::luaL_error(L, lua::lua_tolstring_(L, -1, ptr::null_mut()));
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
        lua::luaL_error(L, cstr!("serialize can't pack too depth table"));
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
    unsafe { lua::lua_pushlstring(L, to_cptr(buff.data()), buff.len()) };
    1
}

pub fn unserialize(L: *mut lua_State) -> int {
    let data = lua::lua_tolstring(L, 1);
    let script = format!("return {:?}", data);
    unsafe {
        if lua::luaL_loadbufferx(L, to_char!(script), script.len(), cstr!("unserialize"), cstr!("bt")) == 0 {
            if lua::lua_pcall(L, 0, 1, 0) == 0 {
                return 1;
            }
        }
        lua::lua_pushnil(L);
        lua::lua_insert(L, -2);
        return 2;
    }
}


// 核心编解码trait
pub trait Codec {
    fn encode(&mut self, L: *mut lua_State, index: i32) -> Result<Vec<u8>, CodecError> {
        Ok(vec![0])
    }
    fn decode(&mut self, L: *mut lua_State) -> Result<Vec<u8>, CodecError>{
        Ok(vec![0])
    }
}

// 基础编解码器实现
pub struct BaseCodec<'a> {
    error: String,
    packet_len: i32,
    buff: *mut LuaBuf,
    slice: Slice<'a>,
}

impl<'a> BaseCodec<'a> {
    pub fn new() -> Self {
        Self {
            packet_len: 0,
            buff: ptr::null_mut(),
            error: "".to_string(),
            slice: Slice::attach(&[]),
        }
    }

    pub fn load_packet(&mut self, data_len: i32) -> i32 {
        if self.slice.is_empty() { return 0; }
        if let Some(packet_len) = self.slice.read::<i32>() {
            if packet_len > 0xffffff { return -1; }
            if packet_len > data_len { return 0; }
            if !self.slice.peek(packet_len as usize, 0).is_some() {
                return 0;
            }
            self.packet_len = packet_len;
            return packet_len;
        }
        0
    }

    pub fn set_buf(&mut self, buf: &mut LuaBuf) {
        self.buff = buf;
    }

    pub fn set_slice(&mut self, slice: Slice<'a>) {
        self.slice = slice;
    }

    pub fn decode_data(&mut self, L: *mut lua_State, data: &'a [u8]) -> Result<Vec<u8>, CodecError> {
        self.slice = Slice::attach(data);
        self.decode(L)
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
}
impl<'a> Codec for BaseCodec<'a> {}

// Lua专用编解码器
pub struct LuaCodec<'a> {
    base: BaseCodec<'a>,
}

impl<'a> LuaCodec<'a> {
    pub fn new() -> Self {
        Self {
            base: BaseCodec::new(),
        }
    }

    pub fn err(&self) -> &str { self.base.err() }
    pub fn failed(&self) -> bool { self.base.failed() }
    pub fn error(&mut self, err: String) { self.base.error(err) }
    pub fn set_buf(&mut self, buf: &mut LuaBuf) { self.base.set_buf(buf) }
    pub fn set_slice(&mut self, slice: Slice<'a>) { self.base.set_slice(slice) }
    pub fn load_packet(&mut self, data_len: i32) -> i32 { self.base.load_packet(data_len) }
    pub fn decode_data(&mut self, L: *mut lua_State, data: &'a [u8]) -> Result<Vec<u8>, CodecError> {
        self.base.decode_data(L, data)
    }
}

impl<'a> Codec for LuaCodec<'a> {}