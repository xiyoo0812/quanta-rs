#![allow(non_snake_case)]
#![allow(dead_code)]

use libc::c_int as int;

use lua::{ cstr, lua_State };

use std::io;
use std::path::Path;
use std::fs::{self, DirEntry};

fn visit_dirs(dir: &Path, cb: &dyn Fn(&DirEntry)) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}