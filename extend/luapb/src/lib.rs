#![allow(improper_ctypes)]
#![allow(non_snake_case)]

extern crate lua;
extern crate libc;
extern crate luakit;

use lua::lua_State;
use libc::c_int as int;

use std::cell::RefCell;
use std::collections::HashMap;

use luakit::{ Luakit, LuaTable, LuaPushFn };

thread_local! {
    static PB_CMD_IDS: RefCell<HashMap<u32, String>> = RefCell::new(HashMap::new());
    static PB_CMD_INDEXS: RefCell<HashMap<String, u32>> = RefCell::new(HashMap::new());
    static PB_CMD_NAMES: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

extern "C" {
    fn luaopen_pb(L: *mut lua_State) -> int;
}

#[no_mangle]
pub extern "C" fn luaopen_luapb(L: *mut lua_State) -> int {
    unsafe { luaopen_pb(L) };
    let mut kit = Luakit::load(L);
    let mut luapb = LuaTable::new(L);
    luakit::set_function!(luapb, "bind_cmd", |cmd_id: u32, name: String, fullname : String| {
        PB_CMD_INDEXS.with_borrow_mut(|indexs: &mut HashMap<String, u32>| {
            indexs.insert(name.clone(), cmd_id);
        });
        PB_CMD_NAMES.with_borrow_mut(|names: &mut HashMap<String, String>| {
            names.insert(name.clone(), fullname);
        });
        PB_CMD_IDS.with_borrow_mut(|ids: &mut HashMap<u32, String>| {
            ids.insert(cmd_id, name);
        });
    });
    kit.set("protobuf", luapb);
    1
}
