#![allow(non_snake_case)]
#![allow(dead_code)]

use std::sync::Arc;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use lua::{ ternary, lua_State };
use std::time::{ SystemTime, UNIX_EPOCH };

//i  - group，10位，(0~1023)
//g  - index，10位(0~1023)
//s  - 序号，13位(0~8912)
//ts - 时间戳，30位
//共63位，防止出现负数

const GROUP_BITS: u32   = 10;
const INDEX_BITS: u32   = 10;
const SNUM_BITS: u32    = 13;

const LETTER_LEN: usize = 12;
const LETTER_SIZE: u64  = 62;

//基准时钟：2022-10-01 08:00:00
const BASE_TIME: u64    = 1664582400;

const MAX_GROUP: u32 = (1 << GROUP_BITS) - 1; // 1023
const MAX_INDEX: u32 = (1 << INDEX_BITS) - 1; // 1023
const MAX_SNUM: u32 = (1 << SNUM_BITS) - 1;   // 8191

//每一group独享一个id生成种子
static LAST_TIME: Lazy<Arc<Mutex<u64>>> = Lazy::new(|| Arc::new(Mutex::new(0)));
static SERIAL_INDEX_TABLE: Lazy<Arc<Mutex<Vec<u32>>>> = Lazy::new(||{
    let indexs : Vec<u32> = vec![0; 1024];
    Arc::new(Mutex::new(indexs))
});

const LETTERS: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

pub fn guid_new(group: u32, index: u32) -> u64 {
    let group = ternary!(group == 0, rand::random::<u32>() % MAX_GROUP, group % MAX_GROUP);
    let index = ternary!(index == 0, rand::random::<u32>() % MAX_GROUP, index % MAX_GROUP);

    let mut serial_index: u32 = 0;
    let mut last_time = LAST_TIME.lock().unwrap();
    let mut serial_index_table = SERIAL_INDEX_TABLE.lock().unwrap();
    let now_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    if now_time > *last_time {
        serial_index_table[group as usize] = 0;
        *last_time = now_time;
    } else {
        serial_index = serial_index_table[group as usize] + 1;
        //种子溢出以后，时钟往前推
        if serial_index >= MAX_SNUM {
            *last_time = now_time + 1;
            serial_index = 0;
        }
        serial_index_table[group as usize] = serial_index;
    }
    ((now_time - BASE_TIME) << (SNUM_BITS + GROUP_BITS + INDEX_BITS))
        | ((serial_index as u64) << (GROUP_BITS + INDEX_BITS))
        | ((index as u64) << GROUP_BITS)
        | (group as u64)
}

pub fn guid_string(group: u32, index: u32) -> String {
    format!("{}", guid_new(group, index))
}

pub fn guid_tostring(guid: u64) -> String {
    format!("{}", guid)
}

pub fn guid_number(guid: String) -> u64 {
    guid.parse::<u64>().unwrap_or(0)
}

pub fn guid_encode(val: u64) -> String {
    let mut val = val;
    let mut tmp = vec!['\0'; LETTER_LEN];
    for i in 0..LETTER_LEN - 1 {
        tmp[i] = LETTERS.chars().nth((val % LETTER_SIZE) as usize).unwrap();
        val /= LETTER_SIZE;
        if val == 0 {
            break;
        }
    }
    tmp.into_iter().collect()
}

fn find_index(val: char) -> u64 {
    match val {
        'a'..='z' => val as u64 - 61,
        'A'..='Z' => val as u64 - 55,
        '0'..='9' => val as u64 - 48,
        _ => 0,
    }
}

pub fn guid_decode(sval: String) -> u64 {
    let mut val = 0u64;
    let len = sval.len();
    let cval = sval.as_bytes();
    for i in 0..len-1 {
        let k = find_index(cval[i] as char);
        val += k * LETTER_SIZE.pow(i as u32);
    }
    val
}

fn format_guid(L: *mut lua_State) -> u64 {
    let ltype = unsafe{ lua::lua_type(L, 1) };
    match ltype {
        lua::LUA_TSTRING => {
            let cval = lua::lua_tolstring(L, 1).unwrap();
            cval.parse::<u64>().unwrap_or(0)
        },
        lua::LUA_TNUMBER => lua::lua_tointeger(L, 1) as u64,
        _ => 0,
    }
}

pub fn guid_group(L: *mut lua_State) -> u32 {
    let guid = format_guid(L);
    (guid & 0x3ff) as u32
}

pub fn guid_index(L: *mut lua_State) -> u32 {
    let guid = format_guid(L);
    ((guid >> GROUP_BITS) & 0x3ff) as u32
}

pub fn guid_time(L: *mut lua_State) -> u64 {
    let guid = format_guid(L);
    ((guid >> (GROUP_BITS + INDEX_BITS + SNUM_BITS)) & 0x3fffffff) + BASE_TIME
}

pub fn guid_source(L: *mut lua_State) -> (u32, u32, u64) {
    let guid = format_guid(L);
    let group = (guid & 0x3ff) as u32;
    let index = ((guid >> GROUP_BITS) & 0x3ff) as u32;
    let time = ((guid >> (GROUP_BITS + INDEX_BITS + SNUM_BITS)) & 0x3fffffff) + BASE_TIME;
    (group, index, time)
}