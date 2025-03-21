#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use std::collections::HashMap;
use once_cell::sync::OnceCell;

use luakit::{LuaBuf, Slice};

const HOLD_OFFSET: u32  = 10;

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
    fn write_varint(&self, buf: &mut LuaBuf) -> Result<(), String>;
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
            fn write_varint(&self, buf: &mut LuaBuf) -> Result<(), String> {
                let mut uval: $u = *self as $u;
                loop {
                    let mut byte = (uval as u8) & 0x7F;
                    uval >>= 7;
                    //设置最高位表示还有后续字节
                    if uval != 0 { byte |= 0x80 }
                    if buf.push_data(&[byte]) == 0 { return Err("length error".to_string()); }
                    if uval == 0 { break }
                }
                Ok(())
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
    value.write_varint(buf)
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

fn write_len_prefixed(buf: &mut LuaBuf, slice: &Slice) -> Result<(), String> {
    write_varint(buf, &slice.size())?;
    if buf.push_data(slice.contents()) > 0 { return Ok(()); }
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

struct PbEnum {
    name: String,
    kvpair: HashMap<i32, String>,
    vkpair: HashMap<String, i32>,
}

impl PbEnum {
    pub fn new(name: String) -> PbEnum {
        PbEnum { name : name, kvpair: HashMap::new(), vkpair: HashMap::new() }
    }
}

struct PbField {
    name: String,
    type_name: String,
    default_value: String,
    number: i32,
    label: i32,
    ftype: FieldType,
    oneof_index: i32,
    penum: Option<PbEnum>,
    message: Option<PbMessage>,
    packed: bool,
}
type PbFieldiMap = HashMap<i32, PbField>;
type PbFieldsMap = HashMap<String, PbField>;

impl PbField {
    pub fn new() -> PbField {
        PbField {
            label: 0,
            number: 0,
            penum: None,
            packed: false,
            message: None,
            oneof_index: 0,
            ftype: FieldType::MAX_TYPE,
            name: "".to_string(),
            type_name: "".to_string(),
            default_value: "".to_string()
        }
    }

    pub fn is_repeated(&mut self) -> bool { self.label == 3 }
    pub fn is_map(&mut self) -> bool { self.get_message().is_some() && self.message.is_map }
    pub fn get_message(&mut self) -> Option<&PbMessage> {
        
     }
}

pub struct PbMessage {
    pub name: String,
    pub fields: PbFieldiMap,
    pub sfields: PbFieldsMap,
    pub oneof_decl: Vec<String>,
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

struct PbDescriptor {
    syntax: String,
    enums: HashMap<String, PbEnum>,
    messages: HashMap<String, PbMessage>
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

static DESCRIPTER: OnceCell<PbDescriptor> = OnceCell::new();
fn Descriptor() -> &'static PbDescriptor {
    DESCRIPTER.get_or_init(|| PbDescriptor::new())
}

