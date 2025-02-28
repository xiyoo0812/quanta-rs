#![allow(non_snake_case)]

extern crate lua;
extern crate libc;
extern crate luakit;

mod excel;

use lua::lua_State;
use libc::c_int as int;

use excel::{ ExcelFile, Sheet, Cell };
use luakit::{Luakit, PtrBox, LuaPush, LuaPushFn, LuaPushLuaFnMut };

fn open_excel(filename: String) -> PtrBox<ExcelFile> {
    let mut excel = ExcelFile::new();
    if !excel.open(filename) {
        return PtrBox::null();
    }
    return PtrBox::new(excel);
}

#[no_mangle]
pub extern "C" fn luaopen_luaxlsx(L: *mut lua_State) -> int {
    let mut kit = Luakit::load(L);
    let mut lxlsx = kit.new_table(Some("xlsx"));
    luakit::set_function!(lxlsx, "open", open_excel);
    luakit::new_class!(Cell, lxlsx, "cell";
        "type"=> typ: String;
        "value"=> value: String
    );
    luakit::new_class!(Sheet, lxlsx, "sheet",
        "get_cell", Sheet::get_cell;
        "last_row"=> last_row: u32;
        "last_col"=> last_col: u32;
        "first_row"=> first_row: u32;
        "first_col"=> first_col: u32;
        "name"=> name: String
    );
    luakit::new_class!(ExcelFile, lxlsx, "excel_file",
        "get_sheet", ExcelFile::get_sheet,
        "sheets", ExcelFile::sheets
    );
    lxlsx.native_to_lua(L)
}
