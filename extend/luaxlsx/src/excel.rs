#![allow(non_snake_case)]
#![allow(dead_code)]

use lua::lua_State;
use libc::c_int as int;

use luakit::{LuaGc, PtrBox, LuaPush};

use calamine::{ Reader, Xlsx, Data };

#[repr(C)]
pub struct Cell {
    pub typ: String,
    pub value: String,
}

impl LuaGc for Cell {
    fn __gc(&mut self) -> bool { false }
}

impl Cell {    
    fn new(data: &Data) -> Self {
        let (value, typ) = match data {
            Data::Int(i) => (i.to_string(), "number".to_string()),
            Data::Bool(b) => (b.to_string(), "bool".to_string()),
            Data::String(s) => (s.clone(), "string".to_string()),
            Data::Float(f) => (f.to_string(), "number".to_string()),
            Data::DateTimeIso(b) => (b.to_string(), "string".to_string()),
            Data::DurationIso(b) => (b.to_string(), "string".to_string()),
            Data::DateTime(b) => (b.to_string(), "string".to_string()),
            Data::Empty => ("".to_string(), "blank".to_string()),
            _ => ("".to_string(), "blank".to_string()),
        };
        Cell { typ, value }
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

    pub fn add_cell(&mut self, row: usize, col: usize, data: &Data) {
        if row < self.first_row || row > self.last_row || col < self.first_col || col > self.last_col{
            return;
        }
        let idx = (row - 1) * (self.last_col - self.first_col + 1) + (col - self.first_col);
        self.cells[idx] = PtrBox::new(Cell::new(data));
    }
}

pub struct ExcelFile {
    sheets: Vec<PtrBox<Sheet>>
}

impl LuaGc for ExcelFile {}
impl Drop for ExcelFile {
    fn drop(&mut self) {
        for sheet in self.sheets.iter() {
            sheet.clone().unwrap();
        }
    }
}

impl ExcelFile {
    pub fn new() -> Self {
        ExcelFile { sheets: Vec::new() }
    }


    pub fn open(&mut self, path: String) -> bool { 
        let mut workbook: Xlsx<_> = match calamine::open_workbook(&path) {
            Ok(wb) => wb,
            Err(_) => return false
        };
        for name in workbook.sheet_names().to_owned() {
            let mut sheet = Sheet::new(&name);
            if let Ok(range) = workbook.worksheet_range(&name) {
                let (nrow, ncol) = range.get_size();
                sheet.first_row = 1;
                sheet.first_col = 1;
                sheet.last_col = ncol;
                sheet.last_row = nrow;
                sheet.cells.resize(nrow * ncol, PtrBox::null());
                for irow in 1..=nrow {
                    for icol in 1..=ncol {
                        if let Some(data) = range.get((irow - 1, icol - 1)) {
                            sheet.add_cell(irow, icol, data);
                        }
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
