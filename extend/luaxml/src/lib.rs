#![allow(non_snake_case)]

extern crate lua;
extern crate libc;
extern crate luakit;

use libc::c_int as int;

use lua::{lua_State, ternary, to_char, to_string, to_utf8};

use luakit::{LuaPush, LuaPushLuaFn, Luakit };
use xmltree::{Element, EmitterConfig, XMLNode};
use std::{collections::HashMap, fs::File, io::{BufReader, Write}};

unsafe fn push_elem2lua(L: *mut lua_State, elem: &Element) {
    let attr = &elem.attributes;
    let children = &elem.children;
    if children.is_empty() && attr.is_empty() {
        lua::lua_pushstring(L, "");
        return;
    }
    lua::lua_createtable(L, 0, 4);
    if let Some(value) = elem.children.iter().find(|e| e.as_text() != None) {
        if let Some(text) = value.as_text() {
            if !lua::lua_stringtonumber(L, text) {
                lua::lua_pushstring(L, text);
            }
        } else {
            lua::lua_pushstring(L, "");
        }
        lua::lua_setfield(L, -2, to_char!("__text"));
    }
    if !attr.is_empty() {
        lua::lua_createtable(L, 0, 4);
        for (key, value) in attr.iter() {
            if !lua::lua_stringtonumber(L, value) {
                lua::lua_pushstring(L, value);
            }
            lua::lua_setfield(L, -2, to_char!(key));
        }
        lua::lua_setfield(L, -2, to_char!("__attr"));
    }
    if !children.is_empty() {
        let mut childs: HashMap<String, Vec<&Element>> = HashMap::new();
        for child in children {
            if let Some(ch_elem) = child.as_element() {
                if let Some(items) = childs.get_mut(&ch_elem.name) {
                    items.push(ch_elem);
                } else {
                    childs.insert(ch_elem.name.clone(), vec![ch_elem]);
                }
            }
        }
        for (name, items) in childs {
            if items.len() == 1 {
                push_elem2lua(L, items[0]);
            } else {
                lua::lua_createtable(L, 0, 4);
                for i in 0..items.len() {
                    push_elem2lua(L, items[i]);
                    lua::lua_seti(L, -2, (i + 1) as isize);
                }
            }
            lua::lua_setfield(L, -2, to_char!(name));
        }
    }
}

fn decode_xml(L: *mut lua_State, xml: String) ->int {
    let root = match Element::parse(xml.as_bytes()) {
        Ok(root) => root,
        Err(e) => {
            unsafe{ lua::lua_pushnil(L) };
            lua::lua_pushstring(L,  &e.to_string());
            return 2;
        }
    };
    unsafe { 
        lua::lua_createtable(L, 0, 4);
        push_elem2lua(L, &root);
        lua::lua_setfield(L, -2, to_char!(root.name)) 
    }
    return 1;
}

fn open_xml(L: *mut lua_State, xmlfile: String) ->int {
    if let Ok(file) = File::open(&xmlfile) {
        let reader = BufReader::new(file);
        let root = match Element::parse(reader) {
            Ok(root) => root,
            Err(e) => {
                unsafe{ lua::lua_pushnil(L) };
                lua::lua_pushstring(L,  &e.to_string());
                return 2;
            }
        };
        unsafe { 
            lua::lua_createtable(L, 0, 4);
            push_elem2lua(L, &root);
            lua::lua_setfield(L, -2, to_char!(root.name));
            return 1;
        }
    }
    unsafe{ lua::lua_pushnil(L) };
    lua::lua_pushstring(L, "open xml file failed!");
    2
}


unsafe fn load_table4lua(L: *mut lua_State, stack: &mut Vec<Element>) {
    // let _gl = LuaGuard::new(L);
    let elem = stack.last_mut().unwrap();
    if lua::lua_getfield(L, -1, to_char!("__attr")) == lua::LUA_TTABLE {
        lua::lua_pushnil(L);
        while lua::lua_next(L, -2) != 0 {
            let key = to_utf8(lua::lua_tostring(L, -2));
            match lua::lua_type(L, -1) {
                lua::LUA_TSTRING => { elem.attributes.insert(key, to_utf8(lua::lua_tostring(L, -1))); },
                lua::LUA_TBOOLEAN => { elem.attributes.insert(key, to_utf8(lua::lua_tostring(L, -1))); },
                lua::LUA_TNUMBER => { elem.attributes.insert(key, to_utf8(lua::lua_tostring(L, -1))); },
                _ => {},
            }
            lua::lua_pop(L, 1);
        }
    }
    lua::lua_pop(L, 1);
    lua::lua_pushnil(L);
    lua::lua_setfield(L, -2, to_char!("__attr"));
    match lua::lua_getfield(L, -1, to_char!("__text")) {
        lua::LUA_TSTRING => elem.children.push(XMLNode::Text(to_utf8(lua::lua_tostring(L, -1)))),
        lua::LUA_TNUMBER => elem.children.push(XMLNode::Text(to_utf8(lua::lua_tostring(L, -1)))),
        lua::LUA_TBOOLEAN => elem.children.push(XMLNode::Text(ternary!(lua::lua_toboolean(L, -1), "true".to_string(), "false".to_string()))),
        _ => {},
    };
    lua::lua_pop(L, 1);
    lua::lua_pushnil(L);
    lua::lua_setfield(L, -2, to_char!("__text"));
    lua::lua_pushnil(L);
    while lua::lua_next(L, -2) != 0 {
        load_elem4lua(L, stack);
        lua::lua_pop(L, 1);
    }
}


unsafe fn load_elem4lua(L: *mut lua_State, stack: &mut Vec<Element>) {
    let key: String = to_utf8(lua::lua_tostring(L, -2));
    if !luakit::is_lua_array(L, -1, false) {
        stack.push(Element::new(&key));
        let elem = stack.last_mut().unwrap();
        match lua::lua_type(L, -1) {
            lua::LUA_TTABLE => load_table4lua(L, stack),
            lua::LUA_TSTRING => elem.children.push(XMLNode::Text(to_utf8(lua::lua_tostring(L, -1)))),
            lua::LUA_TNUMBER => elem.children.push(XMLNode::Text(to_utf8(lua::lua_tostring(L, -1)))),
            lua::LUA_TBOOLEAN => elem.children.push(XMLNode::Text(ternary!(lua::lua_toboolean(L, -1), "true".to_string(), "false".to_string()))),
            tp => lua::luaL_error(L, format!("xml encode not support type:{}!", to_string(lua::lua_typename(L, tp))).as_str()),
        };
        if stack.len() > 1 {
            let cur = stack.pop().unwrap();
            stack.last_mut().unwrap().children.push(XMLNode::Element(cur));
        }
        return;
    }
    
    lua::lua_pushstring(L, &key);
    let raw_len = lua::lua_rawlen(L, -2);
    for i in 1..=raw_len {
        lua::lua_rawgeti(L, -2, i as i32);
        load_elem4lua(L, stack);
        lua::lua_pop(L, 1);
    }
    lua::lua_pop(L, 1);
}

fn encode_xml(L: *mut lua_State) -> int {
    unsafe {
        lua::lua_pushnil(L);
        if lua::lua_next(L, 1) != 0 {
            let mut buffer = Vec::new();
            let mut stack: Vec<_> = Vec::new();
            load_elem4lua(L, &mut stack);
            let declaration = lua::luaL_optstring(L, 2);
            if !declaration.is_empty() {
                let _ = buffer.write(format!("<?{}?>\r\n", to_utf8(declaration)).as_bytes());
            }
            let config = EmitterConfig::new()
                .line_separator("\r\n")
                .perform_indent(true)
                .write_document_declaration(declaration.is_empty());
            if let Ok(_) = stack[0].write_with_config(&mut buffer, config) {
                lua::lua_pushstring(L, &to_utf8(&buffer));
                return 1;
            }
        }
    }
    lua::luaL_error(L, "xml encode failed");
}

fn save_xml(L: *mut lua_State, xmlfile: String) -> int {
    unsafe {
        if let Ok(mut file) = File::create(&xmlfile) {
            lua::lua_pushnil(L);
            if lua::lua_next(L, 2) != 0 {
                let mut stack: Vec<_> = Vec::new();
                load_elem4lua(L, &mut stack);
                let declaration = lua::luaL_optstring(L, 3);
                if !declaration.is_empty() {
                    let _ = file.write(format!("<?{}?>\r\n", to_utf8(declaration)).as_bytes());
                }
                let config = EmitterConfig::new()
                    .line_separator("\r\n")
                    .perform_indent(true)
                    .write_document_declaration(declaration.is_empty());
                if let Ok(_) = stack[0].write_with_config(&mut file, config) {
                    lua::lua_pushboolean(L, 1);
                    return 1;
                }
            }
        }
    }
    lua::luaL_error(L, "xml save failed");
}

#[no_mangle]
pub extern "C" fn luaopen_luaxml(L: *mut lua_State) -> int {
    let mut kit = Luakit::load(L);
    let mut lxml = kit.new_table(Some("xml"));
    luakit::set_function!(lxml, "decode", decode_xml);
    luakit::set_function!(lxml, "encode", encode_xml);
    luakit::set_function!(lxml, "open", open_xml);
    luakit::set_function!(lxml, "save", save_xml);
    lxml.native_to_lua(L)
}
