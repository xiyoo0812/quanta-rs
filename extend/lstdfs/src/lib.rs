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
    let LFUNS = [
        lua_reg!("dir", lstdfs_dir),
        lua_reg!("stem", lstdfs_stem),
        lua_reg!("copy", lstdfs_copy),
        lua_reg!("mkdir", lstdfs_mkdir),
        lua_reg!("chdir", lstdfs_chdir),
        lua_reg!("split", lstdfs_split),
        lua_reg!("rename", lstdfs_rename),
        lua_reg!("exists", lstdfs_exists),
        lua_reg!("remove", lstdfs_remove),
        lua_reg!("append", lstdfs_append),
        lua_reg!("concat", lstdfs_concat),
        lua_reg!("temp_dir", lstdfs_temp_dir),
        lua_reg!("absolute", lstdfs_absolute),
        lua_reg!("relative", lstdfs_relative),
        lua_reg!("filetype", lstdfs_filetype),
        lua_reg!("filename", lstdfs_filename),
        lua_reg!("copy_file", lstdfs_copy_file),
        lua_reg!("file_size", lstdfs_file_size),
        lua_reg!("extension", lstdfs_extension),
        lua_reg!("root_name", lstdfs_root_name),
        lua_reg!("root_path", lstdfs_root_path),
        lua_reg!("parent_path", lstdfs_parent_path),
        lua_reg!("remove_file", lstdfs_remove_file),
        lua_reg!("is_absolute", lstdfs_is_absolute),
        lua_reg!("is_directory", lstdfs_is_directory),
        lua_reg!("current_path", lstdfs_current_path),
        lua_reg!("relative_path", lstdfs_relative_path),
        lua_reg!("make_preferred", lstdfs_make_preferred),
        lua_reg!("last_write_time", lstdfs_last_write_time),
        lua_reg!("remove_filename", lstdfs_remove_filename),
        lua_reg!("replace_filename", lstdfs_replace_filename),
        lua_reg!("replace_extension", lstdfs_replace_extension),
        lua_reg!(),
    ];
    unsafe {
        lua::lua_createtable(L, LFUNS.len() as i32, 0);
        lua::luaL_setfuncs(L, LFUNS.as_ptr(), 0);
        return 1;
    }
}
