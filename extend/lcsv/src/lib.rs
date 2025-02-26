#![allow(non_snake_case)]

extern crate lua;
extern crate libc;
extern crate luakit;

mod csv;

use libc::c_int as int;

use lua::{lua_State, to_char};

use ::csv::{Reader, Writer};
use csv::{ CsvFile, Sheet, Cell };
use luakit::{Luakit, PtrBox, LuaPush, LuaPushFn, LuaPushLuaFn, LuaPushLuaFnMut };

fn decode_value(L: *mut lua_State, key: &str) {
    if !lua::lua_stringtonumber(L, key) {
        lua::lua_pushlstring(L, key);
    }
}

fn decode_reader<T>(L: *mut lua_State, mut reader: Reader<T>) -> int where T: std::io::Read {
    let csv_headers =  reader.headers().unwrap();
    let headers: Vec<String> = csv_headers.iter().map(|s| s.to_string()).collect();

    let mut index = 1;
    let keyidx = lua::luaL_optinteger(L, 2, 0) - 1;
    unsafe { lua::lua_createtable(L, 0, 4) };
    for result in reader.records() {
        if let Err(e) = result {
            lua::luaL_error(L, &format!("csv read records failed: {}", e.to_string()));
        }
        let mut key = "";
        unsafe { lua::lua_createtable(L, 0, 4) };
        let record = result.unwrap();
        let values: Vec<&str> = record.iter().collect();
        for i in 0..values.len() {
            if i < headers.len() {
                if (i as isize) == keyidx {
                    key = &values[i]
                }
                decode_value(L, &headers[i]);
                decode_value(L, values[i]);
                unsafe { lua::lua_rawset(L, -3) };
            }
        }
        if key.is_empty() {
            unsafe { lua::lua_seti(L, -2, index) };
            index += 1;
        }
        else {
            decode_value(L, key);
            lua::lua_insert(L, -2);
            unsafe { lua::lua_rawset(L, -3) };
        }
    }
    return 1;
}

fn decode_csv(L: *mut lua_State, doc: String) ->int {
    let reader = Reader::from_reader(doc.as_bytes());
    return decode_reader(L, reader);
}

fn read_csv(L: *mut lua_State, csvfile: String) ->int {
    match Reader::from_path(csvfile) {
        Ok(reader) => decode_reader(L, reader),
        Err(e) => {
            lua::luaL_error(L, &format!("csv read records failed: {}", e.to_string()));
        }
    }
}

fn encode_value(L: *mut lua_State, index: int) -> String {
    match unsafe { lua::lua_type(L, index) } {
        lua::LUA_TNUMBER=> lua::to_utf8(lua::lua_tostring(L, index)),
        lua::LUA_TSTRING=> lua::to_utf8(lua::lua_tostring(L, index)),
        _ => lua::luaL_error(L, "unsuppert lua type"),
    }
}

fn encode_header(L: *mut lua_State, index: int, header: &mut Vec<String>) {
    unsafe {
        lua::lua_pushnil(L);
        while lua::lua_next(L, index) != 0 {
            header.push(encode_value(L, -2));
            lua::lua_pop(L, 1);
        }
    }
}

fn encode_row(L: *mut lua_State, index: int, header: &Vec<String>) -> Vec<String>  {
    unsafe {
        let mut row = Vec::new();
        if luakit::is_lua_array(L, index, false) {
            let len = lua::lua_rawlen(L, index);
            for i in 1..=len {
                lua::lua_geti(L, index, i as isize);
                row.push(encode_value(L, -1));
                lua::lua_pop(L, 1);
            }
        } else if header.is_empty() {
            lua::lua_pushnil(L);
            while lua::lua_next(L, index) != 0 {
                row.push(encode_value(L, -1));
                lua::lua_pop(L, 1);
            }
        } else {
            for item in header {
                lua::lua_getfield(L, index, to_char!(item));
                row.push(encode_value(L, -1));
                lua::lua_pop(L, 1);
            }
        }
        row
    }
}

fn open_csv(filename: String) -> PtrBox<CsvFile> {
    let mut csv = CsvFile::new();
    if !csv.open(filename) {
        return PtrBox::null();
    }
    return PtrBox::new(csv);
}

fn encode_rows<T>(L: *mut lua_State, writer: &mut Writer<T>) where T: std::io::Write {
    unsafe {
        if !lua::lua_istable(L, -1) {
            lua::luaL_error(L, "csv encode args must be a table");
        }
        lua::lua_pushnil(L);
        let mut header_load = false;
        let mut header = Vec::<String>::new();
        while lua::lua_next(L, -2) != 0 {
            if lua::lua_istable(L, -1) {
                if !luakit::is_lua_array(L, -1, false) && !header_load {
                    header_load = true;
                    encode_header(L, lua::lua_absindex(L, -1), &mut header);
                    let _ = writer.write_record(header.clone());
                }
                let row = encode_row(L, lua::lua_absindex(L, -1), &header);
                let _ = writer.write_record(row.clone());
            }
            lua::lua_pop(L, 1);
        }
    }
}

fn encode_csv(L: *mut lua_State) -> int {
    let mut writer = Writer::from_writer(vec![]);
    encode_rows(L, &mut writer);
    if let Ok(mut inner) = writer.into_inner() {
        unsafe { lua::lua_pushlstring_(L, inner.as_mut_ptr() as *const i8, inner.len() as usize) };
        return 1;
    }
    lua::luaL_error(L, "csv encode failed");
}

fn save_csv(L: *mut lua_State, csvfile: String) -> int {
    if let Ok(mut writer) = Writer::from_path(&csvfile) {
        encode_rows(L, &mut writer);
        unsafe { lua::lua_pushboolean(L, 1) };
        return 1;
    }
    lua::luaL_error(L, "csv encode failed");
}

#[no_mangle]
pub extern "C" fn luaopen_lcsv(L: *mut lua_State) -> int {
    let mut kit = Luakit::load(L);
    let mut lcsv = kit.new_table(Some("csv"));
    luakit::set_function!(lcsv, "decode", decode_csv);
    luakit::set_function!(lcsv, "encode", encode_csv);
    luakit::set_function!(lcsv, "read", read_csv);
    luakit::set_function!(lcsv, "save", save_csv);
    luakit::set_function!(lcsv, "open", open_csv);
    luakit::new_class!(Cell, lcsv, "cell";
        "type"=> typ: String;
        "value"=> value: String
    );
    luakit::new_class!(Sheet, lcsv, "sheet",
        "get_cell", Sheet::get_cell;
        "last_row"=> last_row: u32;
        "last_col"=> last_col: u32;
        "first_row"=> first_row: u32;
        "first_col"=> first_col: u32;
        "name"=> name: String
    );
    luakit::new_class!(CsvFile, lcsv, "csv_file",
        "get_sheet", CsvFile::get_sheet,
        "sheets", CsvFile::sheets
    );
    lcsv.native_to_lua(L)
}
