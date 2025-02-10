#![allow(non_snake_case)]
#![allow(dead_code)]

extern crate lua;
extern crate libc;
extern crate luakit;

mod stdfs;

use stdfs::*;
use lua::lua_State;
use libc::c_int as int;
use luakit::{ Luakit, LuaPush };

#[no_mangle]
pub extern "C" fn luaopen_lstdfs(L: *mut lua_State) -> int {
    let mut kit = Luakit::load(L);
    let mut stdfs = kit.new_table(Some("stdfs"));
    stdfs.set_function("dir", lstdfs_dir);
    stdfs.set_function("stem", lstdfs_stem);
    stdfs.set_function("copy", lstdfs_copy);
    stdfs.set_function("mkdir", lstdfs_mkdir);
    stdfs.set_function("chdir", lstdfs_chdir);
    stdfs.set_function("split", lstdfs_split);
    stdfs.set_function("rename", lstdfs_rename);
    stdfs.set_function("exists", lstdfs_exists);
    stdfs.set_function("remove", lstdfs_remove);
    stdfs.set_function("append", lstdfs_append);
    stdfs.set_function("concat", lstdfs_concat);
    stdfs.set_function("temp_dir", lstdfs_temp_dir);
    stdfs.set_function("absolute", lstdfs_absolute);
    stdfs.set_function("relative", lstdfs_relative);
    stdfs.set_function("filetype", lstdfs_filetype);
    stdfs.set_function("filename", lstdfs_filename);
    stdfs.set_function("copy_file", lstdfs_copy_file);
    stdfs.set_function("file_size", lstdfs_file_size);
    stdfs.set_function("extension", lstdfs_extension);
    stdfs.set_function("root_name", lstdfs_root_name);
    stdfs.set_function("root_path", lstdfs_root_path);
    stdfs.set_function("parent_path", lstdfs_parent_path);
    stdfs.set_function("remove_file", lstdfs_remove_file);
    stdfs.set_function("is_absolute", lstdfs_is_absolute);
    stdfs.set_function("is_directory", lstdfs_is_directory);
    stdfs.set_function("current_path", lstdfs_current_path);
    stdfs.set_function("relative_path", lstdfs_relative_path);
    stdfs.set_function("make_preferred", lstdfs_make_preferred);
    stdfs.set_function("last_write_time", lstdfs_last_write_time);
    stdfs.set_function("remove_filename", lstdfs_remove_filename);
    stdfs.set_function("replace_filename", lstdfs_replace_filename);
    stdfs.set_function("replace_extension", lstdfs_replace_extension);
    stdfs.native_to_lua(L)
}
