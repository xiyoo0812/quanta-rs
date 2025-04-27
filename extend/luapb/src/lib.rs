#![allow(improper_ctypes)]
#![allow(non_snake_case)]

extern crate lua;
extern crate luakit;

mod luapb;

use lua::lua_State;
use libc::c_int as int;

use std::io::Read;
use std::fs::File;
use std::cell::RefCell;
use std::collections::HashMap;

use luakit::{ LuaPushFn, LuaPushLuaFn, Luakit, PtrBox, Slice };
use luapb::{ find_enum, find_message, pb_clear, pb_enums, pb_messages, encode_message, decode_message, read_file_descriptor_set, PbMessage};

thread_local! {
    static PB_CMD_INDEXS: RefCell<HashMap<String, u32>> = RefCell::new(HashMap::new());
    static PB_CMD_IDS: RefCell<HashMap<u32, PtrBox<PbMessage>>> = RefCell::new(HashMap::new());
    static PB_CMD_NAMES: RefCell<HashMap<String, PtrBox<PbMessage>>> = RefCell::new(HashMap::new());
}

fn pbmsg_from_cmdid(cmd_id: u32) -> PtrBox<PbMessage> {
    PB_CMD_IDS.with_borrow(|pb_cmd_ids| {
        if let Some(message) = pb_cmd_ids.get(&cmd_id) {
            return message.clone();
        }
        PtrBox::null()
    })
}

unsafe fn pbmsg_from_stack(L: *mut lua_State, index: i32, cmd_id: &mut u32) -> Result<PtrBox<PbMessage>, String> {
    if lua::lua_isnumber(L, index) != 0 {
        let cmdid = lua::lua_tointeger(L, index) as u32;
        let msg = pbmsg_from_cmdid(cmdid);
        if msg.is_null() {
            return Err(format!("invalid pb cmd: {}", cmdid));
        }
        *cmd_id = cmdid;
        return Ok(msg);
    }
    if lua::lua_isstring(L, index) != 0 {
        let cmd_name = lua::to_utf8(lua::lua_tostring(L, index));
        let msg = PB_CMD_NAMES.with_borrow(|pb_cmd_names| {
            if let Some(message) = pb_cmd_names.get(&cmd_name) {
                return message.clone();
            }
            if *cmd_id == 0 {
                return find_message(&cmd_name);
            }
            PtrBox::null()
        });
        if msg.is_null() {
            return Err(format!("invalid pb cmd: {}", cmd_name));
        }
        if *cmd_id != 0 {
            *cmd_id = PB_CMD_INDEXS.with_borrow(|pb_cmd_indexs| {
                if let Some(index) = pb_cmd_indexs.get(&cmd_name) {
                    return *index;
                }
                return 0;
            });
        }
        return Ok(msg);
    }
    return Err(format!("invalid pb cmd arg type"));
}


fn load_pb(L: *mut lua_State) -> int {
    let data = lua::lua_tolstring(L, 1);
    let buf = luakit::get_buff();
    buf.push_data(data);
    match read_file_descriptor_set(&mut buf.get_slice(Some(0), Some(0))) {
        Ok(()) => unsafe { lua::lua_pushboolean(L, 1) },
        Err(e) => lua::luaL_error(L, &e),
    }
    1
}

fn load_file(L: *mut lua_State, filename: String) -> int {
    if let Ok(mut file) = File::open(filename) {
        let buf = luakit::get_buff();
        let mut bytes = Vec::new();
        if let Ok(_) = file.read_to_end(&mut bytes) {
            buf.push_data(&bytes);
            match read_file_descriptor_set(&mut buf.get_slice(Some(0), Some(0))) {
                Ok(()) => unsafe { lua::lua_pushboolean(L, 1) },
                Err(e) => lua::luaL_error(L, &e),
            }
        }
    }
    0
}

fn pb_encode(L: *mut lua_State) ->int {
    match unsafe { pbmsg_from_stack(L, 1, &mut 0) } {
        Ok(mut msg) => {
            let buf = luakit::get_buff();
            buf.clean();
            match unsafe { encode_message(L, buf, &mut msg) } {
                Ok(()) => lua::lua_pushlstring(L, buf.string()),
                Err(e) => lua::luaL_error(L, &e),
            }
        },
        Err(e) => lua::luaL_error(L, &e),
    }
    1
}

fn pb_decode(L: *mut lua_State) ->int {
    match unsafe { pbmsg_from_stack(L, 1, &mut 0) } {
        Ok(mut msg) => {
            let val = lua::lua_tolstring(L, 2);
            let mut slice = Slice::attach(val);
            match unsafe { decode_message(L, &mut slice, &mut msg) }{
                Ok(()) => {},
                Err(e)=> {
                    lua::luaL_error(L, &e)
                },
            }
        },
        Err(e)=> lua::luaL_error(L, &e),
    }
    1
}

fn pb_enum_id(L: *mut lua_State, efullname: String) -> int {
    let penum = find_enum(&efullname);
    if !penum.is_null() {
        unsafe  {
            if lua::lua_isstring(L, 2) != 0 {
                let key = lua::to_utf8(lua::lua_tostring(L, 2));
                if let Some(val) = penum.kvpair.get(&key) {
                    lua::lua_pushinteger(L, *val as isize);
                    return 1;
                }
            }
            if lua::lua_isnumber(L, 2) != 0 {
                let key = lua::lua_tointeger(L, 2) as i32;
                if let Some(val) = penum.vkpair.get(&key) {
                    lua::lua_pushstring(L, val);
                    return 1;
                }
            }
                
        }
    }
    return 0;
}

#[no_mangle]
pub extern "C" fn luaopen_luapb(L: *mut lua_State) -> int {
    let mut kit = Luakit::load(L);
    let mut luapb = kit.new_table(Some("protobuf"));
    luakit::set_function!(luapb, "load", load_pb);
    luakit::set_function!(luapb, "clear", pb_clear);
    luakit::set_function!(luapb, "enums", pb_enums);
    luakit::set_function!(luapb, "enum", pb_enum_id);
    luakit::set_function!(luapb, "decode", pb_decode);
    luakit::set_function!(luapb, "encode", pb_encode);
    luakit::set_function!(luapb, "loadfile", load_file);
    luakit::set_function!(luapb, "messages", pb_messages);
    //  luakit::set_function("pbcodec", pb_codec);
    luakit::set_function!(luapb, "bind_cmd", |cmd_id: u32, name: String, fullname : String| {
        let message = find_message(&fullname);
        if !message.is_null() {
            PB_CMD_INDEXS.with_borrow_mut(|indexs| {
                indexs.insert(name.clone(), cmd_id);
            });
            PB_CMD_NAMES.with_borrow_mut(|names| {
                names.insert(name.clone(), message.clone());
            });
            PB_CMD_IDS.with_borrow_mut(|ids| {
                ids.insert(cmd_id, message.clone());
            });
        }
        
    });
    1
}
