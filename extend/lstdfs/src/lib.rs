#![allow(non_snake_case)]

extern crate lua;
extern crate libc;
extern crate luakit;

mod stdfs;

use stdfs::*;
use libc::c_int as int;
use lua::{ cstr, lua_reg, lua_State };

#[no_mangle]
pub extern "C" fn luaopen_lstdfs(L: *mut lua_State) -> int {
    let lfuns = [
        lua_reg!("dir", |L| { return lstdfs_dir(L);}),
        lua_reg!("stem", |L| { return lstdfs_stem(L);}),
        lua_reg!("copy", |L| { return lstdfs_copy(L);}),
        lua_reg!("mkdir", |L| { return lstdfs_mkdir(L);}),
        lua_reg!("chdir", |L| { return lstdfs_chdir(L);}),
        lua_reg!("split", |L| { return lstdfs_split(L);}),
        lua_reg!("concat", |L| { return lstdfs_concat(L);}),
        lua_reg!("rename", |L| { return lstdfs_rename(L);}),
        lua_reg!("remove", |L| { return lstdfs_remove(L);}),
        lua_reg!("exists", |L| { return lstdfs_exists(L);}),
        lua_reg!("append", |L| { return lstdfs_append(L);}),
        lua_reg!("copy_file", |L| { return lstdfs_copy(L);}),
        lua_reg!("relative", |L| { return lstdfs_relative(L);}),
        lua_reg!("temp_dir", |L| { return lstdfs_temp_dir(L);}),
        lua_reg!("filename", |L| { return lstdfs_filename(L);}),
        lua_reg!("filetype", |L| { return lstdfs_filetype(L);}),
        lua_reg!("extension", |L| { return lstdfs_extension(L);}),
        lua_reg!("file_size", |L| { return lstdfs_file_size(L);}),
        lua_reg!("is_absolute", |L| { return lstdfs_is_absolute(L);}),
        lua_reg!("parent_path", |L| { return lstdfs_parent_path(L);}),
        lua_reg!("current_path", |L| { return lstdfs_current_path(L);}),
        lua_reg!("is_directory", |L| { return lstdfs_is_directory(L);}),
        lua_reg!("last_write_time", |L| { return lstdfs_last_write_time(L);}),
        lua_reg!(),
    ];
    unsafe {
        lua::lua_createtable(L, 2, 0);
        lua::luaL_setfuncs(L, lfuns.as_ptr(), 0);
        return 1;
    }
}
