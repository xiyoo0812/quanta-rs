#![allow(non_snake_case)]

extern crate lua;
extern crate libc;
extern crate luakit;

mod stdfs;

use stdfs::*;
use libc::c_int as int;
use libc::c_char as char;
use lua::{ cstr, lua_State };
use luakit::{ LuaPush, LuaRead, Luakit };

enum Direction{
    Up,
    Down,
    Left,
    Right,
}

pub struct Test {
    pub i: i32,
}

impl Test {
    fn new() -> Self {
        Test { i : 2}
    }

    fn some_method(&self) -> i32 {
        42
    }
}

// 实现 LuaRead trait 以满足 new_class! 宏的要求
impl LuaRead for Test {
    fn lua_to_native(L: *mut lua_State, _index: int) -> Option<Self> {
        // 实现从 Lua 栈中读取 Test 实例的逻辑
        // 这里只是一个示例，具体实现取决于你的需求
        Some(Test {i:2})
    }
}

#[no_mangle]
pub extern "C" fn luaopen_lstdfs(L: *mut lua_State) -> int {
    let mut kit = Luakit::load(L);
    let mut lstdfs = kit.new_table(Some(cstr!("stdfs")));
    luakit::new_enum!(lstdfs, "Mode",
        "Up", Direction::Up as int,
        "Down", Direction::Down as int,
        "Left", Direction::Left as int,
        "Right", Direction::Right as int
    );
    luakit::new_class!(Test, kit, "Test", 
        "some_method", Test::some_method, "i", Test::i);
    let mut stdfs = luakit::new_enum!(kit, "Direction",
        "Up", Direction::Up as int,
        "Down", Direction::Down as int,
        "Left", Direction::Left as int,
        "Right", Direction::Right as int
    );
    stdfs.set_function(cstr!("dir"), lstdfs_dir);
    stdfs.set_function(cstr!("stem"), lstdfs_stem);
    stdfs.set_function(cstr!("copy"), lstdfs_copy);
    stdfs.set_function(cstr!("mkdir"), lstdfs_mkdir);
    stdfs.set_function(cstr!("chdir"), lstdfs_chdir);
    stdfs.set_function(cstr!("split"), lstdfs_split);
    stdfs.set_function(cstr!("rename"), lstdfs_rename);
    stdfs.set_function(cstr!("exists"), lstdfs_exists);
    stdfs.set_function(cstr!("remove"), lstdfs_remove);
    stdfs.set_function(cstr!("append"), lstdfs_append);
    stdfs.set_function(cstr!("concat"), lstdfs_concat);
    stdfs.set_function(cstr!("temp_dir"), lstdfs_temp_dir);
    stdfs.set_function(cstr!("absolute"), lstdfs_absolute);
    stdfs.set_function(cstr!("relative"), lstdfs_relative);
    stdfs.set_function(cstr!("filetype"), lstdfs_filetype);
    stdfs.set_function(cstr!("filename"), lstdfs_filename);
    stdfs.set_function(cstr!("copy_file"), lstdfs_copy_file);
    stdfs.set_function(cstr!("file_size"), lstdfs_file_size);
    stdfs.set_function(cstr!("extension"), lstdfs_extension);
    stdfs.set_function(cstr!("root_name"), lstdfs_root_name);
    stdfs.set_function(cstr!("root_path"), lstdfs_root_path);
    stdfs.set_function(cstr!("parent_path"), lstdfs_parent_path);
    stdfs.set_function(cstr!("remove_file"), lstdfs_remove_file);
    stdfs.set_function(cstr!("is_absolute"), lstdfs_is_absolute);
    stdfs.set_function(cstr!("is_directory"), lstdfs_is_directory);
    stdfs.set_function(cstr!("current_path"), lstdfs_current_path);
    stdfs.set_function(cstr!("relative_path"), lstdfs_relative_path);
    stdfs.set_function(cstr!("make_preferred"), lstdfs_make_preferred);
    stdfs.set_function(cstr!("last_write_time"), lstdfs_last_write_time);
    stdfs.set_function(cstr!("remove_filename"), lstdfs_remove_filename);
    stdfs.set_function(cstr!("replace_filename"), lstdfs_replace_filename);
    stdfs.set_function(cstr!("replace_extension"), lstdfs_replace_extension);
    return stdfs.native_to_lua(L);
}
