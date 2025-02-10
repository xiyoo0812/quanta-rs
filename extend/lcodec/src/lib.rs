#![allow(non_snake_case)]

extern crate lua;
extern crate libc;
extern crate luakit;

mod url;
mod hash;
mod guid;
mod bitset;

use lua::lua_State;
use luakit::{ Luakit, LuaPush, LuaPushFn, LuaPushLuaFn, LuaPushFnMut };

use crate::bitset::BitSet;

#[no_mangle]
pub extern "C" fn luaopen_lcodec(L: *mut lua_State) -> i32 {
    let mut kit = Luakit::load(L);
    let mut lcodec = kit.new_table(Some("codec"));
    luakit::set_function!(lcodec, "hash_code", hash::hash_code);
    luakit::set_function!(lcodec, "url_encode", url::url_encode);
    luakit::set_function!(lcodec, "url_decode", url::url_decode);
    luakit::set_function!(lcodec, "guid_new", guid::guid_new);
    luakit::set_function!(lcodec, "guid_string", guid::guid_string);
    luakit::set_function!(lcodec, "guid_tostring", guid::guid_tostring);
    luakit::set_function!(lcodec, "guid_number", guid::guid_number);
    luakit::set_function!(lcodec, "guid_encode", guid::guid_encode);
    luakit::set_function!(lcodec, "guid_decode", guid::guid_decode);
    luakit::set_function!(lcodec, "guid_source", guid::guid_source);
    luakit::set_function!(lcodec, "guid_group", guid::guid_group);
    luakit::set_function!(lcodec, "guid_index", guid::guid_index);
    luakit::set_function!(lcodec, "guid_time", guid::guid_time);
    luakit::set_function!(lcodec, "bitset", || Box::new(BitSet::new()));
    luakit::new_class!(BitSet, lcodec, "bitset",
        "get", BitSet::get,
        "set", BitSet::set,
        "hex", BitSet::hex,
        "load", BitSet::load,
        "flip", BitSet::flip,
        "reset", BitSet::reset,
        "check", BitSet::check,
        "binary", BitSet::binary,
        "loadhex", BitSet::loadhex,
        "loadbin", BitSet::loadbin,
        "tostring", BitSet::tostring
    );
    lcodec.native_to_lua(L)
}
