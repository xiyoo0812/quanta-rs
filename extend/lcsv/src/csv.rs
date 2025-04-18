#![allow(non_snake_case)]
#![allow(dead_code)]

use std::path::Path;

use libc::c_int as int;
use lua::{ ternary, lua_State };

use luakit::{LuaGc, PtrBox, LuaPush};

#[repr(C)]
pub struct Cell {
    pub typ: String,
    pub value: String,
}

impl LuaGc for Cell {
    fn __gc(&mut self) -> bool { false }
}

impl Cell {
    fn new(value: &str) -> Self {
        Cell {
            value: value.to_string(),
            typ: ternary!(value.is_empty(), "blank".to_string(), "string".to_string())
        }
    }
}

#[repr(C)]
pub struct Sheet {
    pub name: String,
    pub last_row: usize,
    pub last_col: usize,
    pub first_row: usize,
    pub first_col: usize,
    cells: Vec<PtrBox<Cell>>
}

impl Drop for Sheet {
    fn drop(&mut self) {
        for cell in self.cells.iter() {
            if cell.is_null() { continue; }
            cell.clone().unwrap();
        }
    }
}

impl LuaGc for Sheet {
    fn __gc(&mut self) -> bool { false }
}

impl Sheet {
    fn new(nam: &str) -> Self {
        Sheet {
            cells: Vec::new(),
            name: nam.to_string(),
            last_row: 0, last_col: 0,
            first_row: 0, first_col: 0,
        }
    }

    pub fn get_cell(&self, row: usize, col: usize) -> Option<PtrBox<Cell>> {
        if row < self.first_row || row > self.last_row || col < self.first_col || col > self.last_col{
            return None;
        }
        let idx = (row - 1) * (self.last_col - self.first_col + 1) + (col - self.first_col);
        self.cells.get(idx).cloned()
    }

    pub fn add_cell(&mut self, row: usize, col: usize, value: &str) {
        if row < self.first_row || row > self.last_row || col < self.first_col || col > self.last_col{
            return;
        }
        let idx = (row - 1) * (self.last_col - self.first_col + 1) + (col - self.first_col);
        self.cells[idx] = PtrBox::new(Cell::new(value));
    }
}

pub struct CsvFile {
    sheets: Vec<PtrBox<Sheet>>
}

impl LuaGc for CsvFile {}
impl Drop for CsvFile {
    fn drop(&mut self) {
        for sheet in self.sheets.iter() {
            sheet.clone().unwrap();
        }
    }
}

impl CsvFile {
    pub fn new() -> Self {
        CsvFile { sheets: Vec::new() }
    }

    pub fn open(&mut self, path: String) -> bool {
        let fpath = Path::new(path.as_str());
        if let Ok(mut rdr) = csv::Reader::from_path(&path){
            let mut sheet = Sheet::new(fpath.file_stem().unwrap().to_str().unwrap());
            let headers =  rdr.headers().unwrap();
            let col_num = headers.len();
            if col_num > 0 {
                sheet.first_row = 1;
                sheet.first_col = 1;
                sheet.last_row = 1;
                sheet.last_col = col_num;
                let (mut irow, mut icol) = (1, 1);
                sheet.cells.resize(irow * col_num, PtrBox::null());
                for h in headers.iter() {
                    sheet.add_cell(irow, icol, h);
                    icol += 1;
                }
                for result in rdr.records() {
                    if let Err(_) = result {
                        return false;
                    }
                    irow += 1; icol = 1;
                    sheet.last_row = irow;
                    sheet.cells.resize(irow * col_num, PtrBox::null());
                    for value in result.unwrap().iter() {
                        sheet.add_cell(irow, icol, value);
                        icol += 1;
                    }
                }
            }
            self.sheets.push(PtrBox::new(sheet));
            return true;
        }
        false
    }

    pub fn get_sheet(&self, name: String) -> PtrBox<Sheet> {
        for sheet in self.sheets.iter() {
            if sheet.name == name {
                return sheet.clone();
            }
        }
        PtrBox::null()
    }

    pub fn sheets(&mut self, L: *mut lua_State) -> int {
        luakit::vector_return!(L, self.sheets.clone())
    }
}
