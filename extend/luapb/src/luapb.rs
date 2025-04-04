#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use lua::{ lua_State, to_char };

use std::collections::HashMap;
use once_cell::sync::OnceCell;

use luakit::{LuaBuf, LuaPush, LuaRead, PtrBox, Slice};

const HOLD_OFFSET: usize    = 10;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Wiretype {
    VARINT          = 0,    //int32, int64, uint32, uint64, sint32, sint64, bool, enum
    I64             = 1,    //fixed64, sfixed64, double
    LEN             = 2,    //string, bytes, embedded messages, packed repeated fields
    SGROUP          = 3,    //deprecated
    EGROUP          = 4,    //deprecated
    I32             = 5,    //fixed32, sfixed32, float
}

impl From<u32> for Wiretype {
    fn from(val: u32) -> Wiretype {
        match val {
            0 => Wiretype::VARINT,
            1 => Wiretype::I64,
            2 => Wiretype::LEN,
            3 => Wiretype::SGROUP,
            4 => Wiretype::EGROUP,
            5 => Wiretype::I32,
            _ => Wiretype::SGROUP,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FieldType {
    TYPE_DOUBLE     = 1,   // double
    TYPE_FLOAT      = 2,   // float
    TYPE_INT64      = 3,   // int64/sint64
    TYPE_UINT64     = 4,   // uint64
    TYPE_INT32      = 5,   // int32/sint32
    TYPE_FIXED64    = 6,   // fixed64
    TYPE_FIXED32    = 7,   // fixed32
    TYPE_BOOL       = 8,   // bool
    TYPE_STRING     = 9,   // string
    TYPE_GROUP      = 10,  // group (deprecated)
    TYPE_MESSAGE    = 11,  // message（嵌套消息）
    TYPE_BYTES      = 12,  // bytes
    TYPE_UINT32     = 13,  // uint32
    TYPE_ENUM       = 14,  // enum
    TYPE_SFIXED32   = 15,  // sfixed32
    TYPE_SFIXED64   = 16,  // sfixed64
    TYPE_SINT32     = 17,  // sint32
    TYPE_SINT64     = 18,  // sint64
    MAX_TYPE        = 19   // 枚举最大值
}

const fn pb_tag(fieldnum: u32, wiretype: Wiretype) -> u32 {
    (fieldnum << 3) | ((wiretype as u32) & 0b111)
}

fn wiretype_by_fieldtype(t : FieldType) -> Wiretype {
    match t {
    FieldType::TYPE_FLOAT       => Wiretype::I64,
    FieldType::TYPE_INT64       => Wiretype::I32,
    FieldType::TYPE_UINT64      => Wiretype::VARINT,
    FieldType::TYPE_INT32       => Wiretype::VARINT,
    FieldType::TYPE_FIXED64     => Wiretype::I64,
    FieldType::TYPE_FIXED32     => Wiretype::I32,
    FieldType::TYPE_BOOL        => Wiretype::VARINT,
    FieldType::TYPE_STRING      => Wiretype::LEN,
    FieldType::TYPE_GROUP       => Wiretype::LEN,
    FieldType::TYPE_MESSAGE     => Wiretype::LEN,
    FieldType::TYPE_BYTES       => Wiretype::LEN,
    FieldType::TYPE_UINT32      => Wiretype::VARINT,
    FieldType::TYPE_ENUM        => Wiretype::VARINT,
    FieldType::TYPE_SFIXED32    => Wiretype::I32,
    FieldType::TYPE_SFIXED64    => Wiretype::I64,
    FieldType::TYPE_SINT32      => Wiretype::VARINT,
    FieldType::TYPE_SINT64      => Wiretype::VARINT,
    _                           => Wiretype::EGROUP,
    }
}

// ZigZag 编码解码（32位版本）
fn decode_sint32(val: u32) -> i32 {
    ((val >> 1) as i32) ^ -((val & 1) as i32)
}

fn encode_sint32(val: i32) -> u32 {
    ((val << 1) as u32) ^ ((val < 0) as u32).wrapping_neg()
}

// ZigZag 编码解码（64位版本）
fn decode_sint64(val: u64) -> i64 {
    ((val >> 1) as i64) ^ -((val & 1) as i64)
}

fn encode_sint64(val: i64) -> u64 {
    ((val << 1) as u64) ^ ((val < 0) as u64).wrapping_neg()
}

/// Varint 编码写入 trait
pub trait Varint {
    fn read_varint(&mut self, slice: &mut Slice) -> Result<(), String>;
    fn write_varint(&self) -> Vec<u8>;
}

// 为所有符合要求的整数类型实现 Varint 编码
macro_rules! impl_varint_write {
    ($t:ty, $u:ty) => {
        impl Varint for $t {
            fn read_varint(&mut self, slice: &mut Slice) -> Result<(), String> {
                let mut len = 0;
                let data = slice.data(&mut len);
                if len == 0 { return Err("length error".to_string()); }
                if data[0] & 0x80 == 0 {
                    *self = data[0] as $t;
                    slice.erase(1);
                    return Ok(());
                }
                *self = 0;
                let needed = (size_of::<$u>() * 8 + 6) / 7; // ceil(bits/7)
                for i in 0..needed-1 {
                    if i >= len { return Err("length error".to_string()); }
                    let byte = data[i];
                    if byte & 0x80 == 0 {
                        *self = (((byte as $t) & 0x7F) << (i * 7)) as $t;
                        slice.erase(i + 1);
                        return Ok(());
                   }
                }
                Err("length error".to_string())
            }
            fn write_varint(&self) -> Vec<u8> {
                let mut uval: $u = *self as $u;
                let mut bytes = Vec::new();
                loop {
                    let mut byte = (uval as u8) & 0x7F;
                    uval >>= 7;
                    //设置最高位表示还有后续字节
                    if uval != 0 { byte |= 0x80 }
                    bytes.push(byte);
                    if uval == 0 { break }
                }
                bytes
            }
        }
    };
}

// 为常用整数类型生成实现
impl_varint_write!(i32, u32);
impl_varint_write!(u32, u32);
impl_varint_write!(i64, u64);
impl_varint_write!(u64, u64);
impl_varint_write!(usize, u64);
pub fn write_varint<T: Varint>(buf: &mut LuaBuf, value: &T) -> Result<(), String> {
    if buf.push_data(&value.write_varint()) == 0 {
        return Err("write_varint failed".to_string());
    }
    Ok(())
}
pub fn read_varint<T: Varint>(slice: &mut Slice, value: &mut T) -> Result<(), String> {
    value.read_varint(slice)
}

// 读取固定长度类型
fn read_fixtype<T: Copy + Default>(slice: &mut Slice) -> Result<T, String> {
    match slice.read::<T>() {
        Some(v) => Ok(v),
        None => Err("length error".to_string()),
    }
}

// 写入固定长度类型
fn write_fixtype<T: Sized>(buf: &mut LuaBuf, value: T) {
    buf.write::<T>(&value);
}

fn read_string(slice: &mut Slice) -> Result<String, String> {
    let mut length: u32 = 0;
    read_varint(slice, &mut length)?;
    match slice.erase(length as usize) {
        Some(data) => Ok(String::from_utf8_lossy(data).to_string()),
        None => Err("length error".to_string()),
    }
}

fn write_string(buf: &mut LuaBuf, value: &String) -> Result<(), String> {
    write_varint(buf, &value.len())?;
    if buf.push_data(value.as_bytes()) > 0 { return Ok(()); }
    Err("length error".to_string())
}

fn read_len_prefixed<'a>(slice: &'a mut Slice) -> Result<Slice<'a>, String> {
    let mut length: u32 = 0;
    read_varint(slice, &mut length)?;
    match slice.erase(length as usize) {
        Some(data) => Ok(Slice::attach(data)),
        None => Err("length error".to_string()),
    }
}

fn write_len_prefixed(buf: &mut LuaBuf, hold_base: usize) -> Result<(), String> {
    let dst = hold_base + HOLD_OFFSET;
    let size = buf.size() - dst;
    write_varint(buf, &size)?;
    if buf.copy_within(dst, hold_base, size) > 0 { return Ok(()); }
    Err("length error".to_string())
}

fn write_field_type(buf: &mut LuaBuf, field_num: i32, ftype: FieldType ) -> Result<(), String> {
    let value = (field_num << 3) | ((wiretype_by_fieldtype(ftype) as i32) & 7);
    write_varint(buf, &value)
}

fn skip_field(slice: &mut Slice, field_tag: u32) -> Result<(), String> {
    let  wire_type = field_tag & 0x07;
    match wire_type.into() {
        Wiretype::VARINT => read_varint::<u64>(slice, &mut 0),
        Wiretype::I32 => read_fixtype::<i32>(slice).map(|_| ()),
        Wiretype::I64 => read_fixtype::<i64>(slice).map(|_| ()),
        Wiretype::LEN => read_len_prefixed(slice).map(|_| ()),
        _ => Err("length error".to_string())
    }
}

pub struct PbEnum {
    name: String,
    kvpair: HashMap<String, i32>,
    vkpair: HashMap<i32, String>,
}

impl PbEnum {
    pub fn new(name: String) -> PbEnum {
        PbEnum { name : name, kvpair: HashMap::new(), vkpair: HashMap::new() }
    }
}

struct PbField {
    packed: bool,
    name: String,
    type_name: String,
    default_value: String,
    label: i32,
    number: i32,
    oneof_index: i32,
    ftype: FieldType,
    penum: PtrBox<PbEnum>,
    message: PtrBox<PbMessage>,
}
type PbFieldiMap = HashMap<i32, PtrBox<PbField>>;
type PbFieldsMap = HashMap<String, PtrBox<PbField>>;

impl PbField {
    pub fn new() -> PbField {
        PbField {
            label: 0,
            number: 0,
            packed: false,
            penum: PtrBox::null(),
            message: PtrBox::null(),
            oneof_index: 0,
            ftype: FieldType::MAX_TYPE,
            name: "".to_string(),
            type_name: "".to_string(),
            default_value: "".to_string()
        }
    }

    pub fn is_repeated(&mut self) -> bool { self.label == 3 }
    pub fn is_map(&mut self) -> bool { (!self.get_message().is_null()) && self.message.is_map }
    pub fn get_message(&mut self) -> PtrBox<PbMessage> {
        if !self.message.is_null() { return self.message.clone(); }
        if self.ftype == FieldType::TYPE_MESSAGE && !self.type_name.is_empty(){
            self.message = find_message(&self.type_name);
        }
        return self.message.clone();
    }
}

pub struct PbMessage {
    name: String,
    fields: PbFieldiMap,
    sfields: PbFieldsMap,
    oneof_decl: Vec<String>,
    pub is_map: bool
}

impl PbMessage {
    pub fn new() -> PbMessage {
        PbMessage {
            name: "".to_string(),
            fields: HashMap::new(),
            sfields: HashMap::new(),
            oneof_decl: Vec::new(),
            is_map: false
        }
    }
}

impl Drop for PbMessage {
    fn drop(&mut self) {
        self.sfields.clear();
        for (_, field) in self.fields.iter_mut() {
            field.clone().unwrap();
        }
    }
}

struct PbDescriptor {
    syntax: String,
    enums: HashMap<String, PtrBox<PbEnum>>,
    messages: HashMap<String, PtrBox<PbMessage>>
}

impl PbDescriptor {
    pub fn new() -> PbDescriptor {
            PbDescriptor {
            enums: HashMap::new(),
            messages: HashMap::new(),
            syntax: "proto3".to_string()
        }
    }
}

impl Drop for PbDescriptor {
    fn drop(&mut self) {
        for (_, enump) in self.enums.iter_mut() {
            enump.clone().unwrap();
        }
        for (_, message) in self.messages.iter_mut() {
            message.clone().unwrap();
        }
        self.enums.clear();
        self.messages.clear();
    }
}

static DESCRIPTER: OnceCell<PbDescriptor> = OnceCell::new();
fn Descriptor() -> &'static PbDescriptor {
    DESCRIPTER.get_or_init(|| PbDescriptor::new())
}

pub fn find_message(name: &str) -> PtrBox<PbMessage> {
    if let Some(msg) = Descriptor().messages.get(name) {
        return msg.clone();
    }
    PtrBox::null()
}

pub fn find_enum(name: &str) -> PtrBox<PbEnum> {
    if let Some(penum) = Descriptor().enums.get(name) {
        return penum.clone();
    }
    PtrBox::null()
}

fn find_field(message: &PbMessage, field_name: &str) -> PtrBox<PbField> {
    if let Some(field) = message.sfields.get(field_name) {
        return field.clone();
    }
    PtrBox::null()
}

fn find_field_by_number(message: &PbMessage, field_num: i32) -> PtrBox<PbField> {
    if let Some(field) = message.fields.get(&field_num) {
        return field.clone();
    }
    PtrBox::null()
}

fn find_field_by_tag(message: &PbMessage, field_tag: i32) -> PtrBox<PbField> {
    find_field_by_number(message, field_tag >> 3)
}

fn to_boolean(val: i32) -> bool { val > 0 }
fn from_boolean(val: bool) -> i32 { if val { 1 } else { 0} }

macro_rules! read_fixtype2lua {
    ($ftype:ident, $slice:expr, $L:expr) => ({
        let val = read_fixtype::<$ftype>($slice)?;
        val.native_to_lua($L);
        Ok(())
    });
    ($ftype:ident, $func:ident, $slice:expr, $L:expr) => ({
        let val = read_fixtype::<$ftype>($slice)?;
        let fval = $func(val);
        fval.native_to_lua($L);
        Ok(())
    });
}


macro_rules! read_varint2lua {
    ($ftype:ident, $slice:expr, $L:expr) => ({
        let mut val: $ftype = 0;
        read_varint::<$ftype>($slice, &mut val)?;
        val.native_to_lua($L);
        Ok(())
    });
    ($ftype:ident, $func:ident, $slice:expr, $L:expr) => ({
        let mut val: $ftype = 0;
        read_varint::<$ftype>($slice, &mut val)?;
        let fval = $func(val);
        fval.native_to_lua($L);
        Ok(())
    });
}

fn decode_field(L: *mut lua_State, slice: &mut Slice, field: &mut PbField) -> Result<(), String> {
    match field.ftype {
        FieldType::TYPE_FLOAT => read_fixtype2lua!(f32, slice, L),
        FieldType::TYPE_DOUBLE => read_fixtype2lua!(f64, slice, L),
        FieldType::TYPE_FIXED32 => read_fixtype2lua!(i32, slice, L),
        FieldType::TYPE_FIXED64 => read_fixtype2lua!(i64, slice, L),
        FieldType::TYPE_INT32 => read_varint2lua!(i32, slice, L),
        FieldType::TYPE_UINT32 => read_varint2lua!(u32, slice, L),
        FieldType::TYPE_INT64 => read_varint2lua!(i64, slice, L),
        FieldType::TYPE_UINT64 => read_varint2lua!(u64, slice, L),
        FieldType::TYPE_BOOL => read_varint2lua!(i32, to_boolean, slice, L),
        FieldType::TYPE_SINT32 => read_varint2lua!(u32, decode_sint32, slice, L),
        FieldType::TYPE_SINT64 => read_varint2lua!(u64, decode_sint64, slice, L),
        FieldType::TYPE_SFIXED32 => read_fixtype2lua!(u32, decode_sint32, slice, L),
        FieldType::TYPE_SFIXED64 => read_fixtype2lua!(u64, decode_sint64, slice, L),
        FieldType::TYPE_ENUM => read_varint2lua!(i32, slice, L),
        FieldType::TYPE_BYTES | FieldType::TYPE_STRING => {
            let s = read_string(slice)?;
            s.native_to_lua(L);
            Ok(())
        },
        FieldType::TYPE_MESSAGE => {
            let mut mslice = read_len_prefixed(slice)?;
            let mut message = field.get_message();
            unsafe { decode_message(L, &mut mslice, &mut message)? };
            Ok(())
        }
        _ => Err("struct error".to_string()),
    }
}

unsafe fn decode_map(L: *mut lua_State, slice: &mut Slice, field: &mut PbField) -> Result<(), String> {
    let name = to_char!(field.name);
    lua::lua_getfield(L, -1, name);
    if !lua::lua_istable(L, -1) {
        lua::lua_pop(L, 1);
        lua::lua_createtable(L, 0, 4);
        lua::lua_pushvalue(L, -1);
        lua::lua_setfield(L, -3, name);
    }
    let msg = field.get_message();
    let mut mslice = read_len_prefixed(slice)?;
    while !mslice.is_empty() {
        let mut tag: i32 = 0;
        read_varint::<i32>(&mut mslice, &mut tag)?;
        let mut kvfield = find_field_by_number(&msg, tag);
        let number = kvfield.number;
        decode_field(L, &mut mslice, &mut kvfield)?;
        if number == 2 {
            lua::lua_rawset(L, -3);
        }
    }
    lua::lua_pop(L, 1);
    Ok(())
}

unsafe fn decode_repeated(L: *mut lua_State, slice: &mut Slice, field: &mut PbField) -> Result<(), String> {
    let mut len = 1;
    lua::lua_createtable(L, 0, 4);
    let mut rslice = read_len_prefixed(slice)?;
    while !rslice.is_empty() {
        decode_field(L, &mut rslice, field)?;
        lua::lua_rawseti(L, -2, len);
        len += 1;
    }
    lua::lua_setfield(L, -2, to_char!(field.name));
    Ok(())
}

pub unsafe fn decode_message(L: *mut lua_State, slice: &mut Slice, msg: &mut PbMessage) -> Result<(), String> {
    lua::lua_createtable(L, 0, msg.fields.len() as i32);
    while !slice.is_empty() {
        let mut tag: i32 = 0;
        read_varint::<i32>(slice, &mut tag)?;
        let mut field = find_field_by_number(msg, tag);
        if !field.is_null() {
            skip_field(slice, tag as u32)?;
            continue;
        }
        if field.is_map() {
            decode_map(L, slice, &mut field)?;
            continue;
        }
        if field.is_repeated() {
            decode_repeated(L, slice, &mut field)?;
            continue;
        }
        let name: *const i8 = to_char!(field.name);
        decode_field(L, slice, &mut field)?;
        //oneof名字引用
        let oneof_index = field.oneof_index;
        if oneof_index < 0 {
            lua::lua_setfield(L, -2, name);
        } else {
            lua::lua_setfield(L, -2, name);
            lua::lua_pushstring(L, &field.name);
            lua::lua_setfield(L, -2, to_char!(msg.oneof_decl[oneof_index as usize]));
        }
    }
    Ok(())
}

macro_rules! write_fixtype4lua {
    ($ftype:ident, $buf:expr, $L:expr, $index:expr) => ({
        let val = $ftype::lua_to_native($L,  $index).unwrap_or(0 as $ftype);
        write_fixtype($buf, val);
        Ok(())
    });
    ($ftype:ident, $func:ident, $buf:expr, $L:expr, $index:expr) => ({
        let val = $ftype::lua_to_native($L,  $index).unwrap_or(0 as $ftype);
        let fval = $func(val);
        write_fixtype($buf, fval);
        Ok(())
    });
}

macro_rules! write_varint4lua {
    ($ftype:ident, $buf:expr, $L:expr, $index:expr) => ({
        let val = $ftype::lua_to_native($L,  $index).unwrap_or(0 as $ftype);
        write_varint($buf, &val)?;
        Ok(())
    });
    ($ftype:ident, $func:ident, $buf:expr, $L:expr, $index:expr, $def:expr) => ({
        let val = $ftype::lua_to_native($L,  $index).unwrap_or($def as $ftype);
        let fval = $func(val);
        write_varint($buf, &fval)?;
        Ok(())
    });
}

fn encode_field(L: *mut lua_State,  buf: &mut LuaBuf, field: &mut PbField, index: i32) -> Result<(), String> {
    match field.ftype {
        FieldType::TYPE_FLOAT => write_fixtype4lua!(f32, buf, L, index),
        FieldType::TYPE_DOUBLE => write_fixtype4lua!(f64, buf, L, index),
        FieldType::TYPE_FIXED32 => write_fixtype4lua!(i32, buf, L, index),
        FieldType::TYPE_FIXED64 => write_fixtype4lua!(i64, buf, L, index),
        FieldType::TYPE_INT32 => write_varint4lua!(i32, buf, L, index),
        FieldType::TYPE_UINT32 => write_varint4lua!(u32, buf, L, index),
        FieldType::TYPE_INT64 => write_varint4lua!(i64, buf, L, index),
        FieldType::TYPE_UINT64 => write_varint4lua!(u64, buf, L, index),
        FieldType::TYPE_ENUM => write_varint4lua!(i32, buf, L, index),
        FieldType::TYPE_BOOL => write_varint4lua!(bool, from_boolean, buf, L, index, false),
        FieldType::TYPE_SINT32 => write_varint4lua!(i32, encode_sint32, buf, L, index, 0),
        FieldType::TYPE_SINT64 => write_varint4lua!(i64, encode_sint64, buf, L, index, 0),
        FieldType::TYPE_SFIXED32 => write_fixtype4lua!(i32, encode_sint32, buf, L, index),
        FieldType::TYPE_SFIXED64 => write_fixtype4lua!(i64, encode_sint64, buf, L, index),
        FieldType::TYPE_BYTES | FieldType::TYPE_STRING => {
            let val = String::lua_to_native(L, index).unwrap_or("".to_string());
            write_string(buf, &val)?;
            Ok(())
        },
        FieldType::TYPE_MESSAGE => {
            let base = buf.hold_place(HOLD_OFFSET);
            let mut message = field.get_message();
            unsafe { encode_message(L, buf, &mut message)? };
            write_len_prefixed(buf, base)?;
            Ok(())
        }
        _ => Err("struct error".to_string()),
    }
}

unsafe fn encode_map(L: *mut lua_State,  buf: &mut LuaBuf, field: &mut PbField, index: i32) -> Result<(), String> {
    let message = field.get_message();
    let index = lua::lua_absindex(L, index);
    lua::lua_pushnil(L);
    while lua::lua_next(L, index) != 0 {
        write_field_type(buf, field.number, field.ftype)?;
        let base = buf.hold_place(HOLD_OFFSET);
        let mut kfield = find_field_by_number(&message, 1);
        write_field_type(buf, kfield.number, kfield.ftype)?;
        encode_field(L, buf, &mut kfield, -2)?;
        let mut vfield = find_field_by_number(&message, 2);
        write_field_type(buf, vfield.number, vfield.ftype)?;
        encode_field(L, buf, &mut vfield, -1)?;
        write_len_prefixed(buf, base)?;
        lua::lua_pop(L, 1);
    }
    Ok(())
}

unsafe fn encode_repeated(L: *mut lua_State,  buf: &mut LuaBuf, field: &mut PbField, index: i32) -> Result<(), String> {
    write_field_type(buf, field.number, FieldType::TYPE_MESSAGE)?;
    let base = buf.hold_place(HOLD_OFFSET);
    let index = lua::lua_absindex(L, index);
    lua::lua_pushnil(L);
    while lua::lua_next(L, index) != 0 {
        encode_field(L, buf, field, -1)?;
        lua::lua_pop(L, 1);
    }
    write_len_prefixed(buf, base)?;
    Ok(())
}

pub unsafe fn encode_message(L: *mut lua_State,  buf: &mut LuaBuf, msg: &mut PbMessage) -> Result<(), String> {
    lua::lua_pushnil(L);
    let mut oneofencode = false;
    while lua::lua_next(L, -2) != 0 {
        if lua::lua_isstring(L, -2) != 0 {
            let name = String::lua_to_native(L, -2).unwrap_or("".to_string());
            let mut field = find_field(msg, &name);
            if !field.is_null() {
                if field.is_map() {
                    encode_map(L, buf, &mut field, -1)?;
                } else if field.is_repeated() {
                    encode_repeated(L, buf, &mut field, -1)?;
                } else {
                    //oneof处理, 编码一个
                    if field.oneof_index >= 0 {
                        if oneofencode {
                            lua::lua_pop(L, 1);
                            continue;
                        }
                        oneofencode = true;
                    }
                    write_field_type(buf, field.number, field.ftype)?;
                    encode_field(L, buf, &mut field, -1)?;
                }
            }
        }
        lua::lua_pop(L, 1);
    }
    Ok(())
}

fn read_enum_value(slice: &mut Slice, info: &mut PbEnum) -> Result<(), String> {
    let mut value: i32 = 0;
    let pslice = read_len_prefixed(slice)?;
    while !pslice.is_empty() {
        let mut tag: u32 = 0;
        read_varint(&mut pslice, &mut tag)?;
        match tag {
            pb_tag(1, Wiretype::LEN) => info.name = read_string(&mut pslice)?,
            pb_tag(2, Wiretype::VARINT) => read_varint::<i32>(&mut pslice, &mut value)?,
            _ => skip_field(&mut pslice, tag)?,
        }
    }
    info.kvpair.insert(info.name, value);
    info.vkpair.insert(value, info.name);
    Ok(())
}