#![allow(non_snake_case)]
#![allow(dead_code)]

use std::{io, env, fs};

use std::fs::Metadata;
use std::os::windows::fs::MetadataExt;
use std::path::{ self, Path, PathBuf, Component };
use std::path::MAIN_SEPARATOR;

use libc::c_int as int;
use lua::{ cstr, lua_State };
use luakit::{ Luakit, LuaPush };

struct FileEntry {
    ename: String,
    etype: String
}

impl LuaPush for FileEntry {
    fn native_to_lua(self, L: *mut lua_State) -> int {
        unsafe {
            lua::lua_createtable(L, 0, 2);
            self.ename.native_to_lua(L);
            lua::lua_setfield(L, -2, cstr!("name"));
            self.etype.native_to_lua(L);
            lua::lua_setfield(L, -2, cstr!("type"));
            1
        }
    }
}

macro_rules! metadata_find {
    ($metares:ident, $func:ident, $def:expr) => {
        if let Ok(meta) = $metares { meta.$func() } else {  $def }
    };
}

fn get_file_type(metares : &io::Result<Metadata>) -> &'static str {
    if let Ok(meta) = metares {
        if meta.is_file() {
            return "regular";
        } else if meta.is_dir() {
            return "directory";
        } else if meta.is_symlink() {
            return "symlink";
        }
    }
    return "implementation-defined";
}

fn get_relative_path(base: String, path: String) -> Option<PathBuf> {
    let fbase = Path::new(base.as_str());
    let fpath: &Path = Path::new(path.as_str());
    if fpath.starts_with(fbase) {
        fpath.strip_prefix(fbase).ok().map(|p| Path::new("../").join(p).to_path_buf())
    } else {
        fbase.strip_prefix(fpath).ok().map(|p| p.to_path_buf())
    }
}

fn visit_dirs(dir: &Path, recursive: bool, files: &mut Vec<FileEntry>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                files.push(FileEntry {
                    etype: get_file_type(&entry.metadata()).to_string(),
                    ename: entry.file_name().into_string().unwrap()
                });
                let path = entry.path();
                if path.is_dir() && recursive {
                    visit_dirs(&path, recursive, files);
                }
            }
        }
    }
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            copy_dir_all(path, dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(path, dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

pub fn lstdfs_dir(L: *mut lua_State) -> int {
    let path = lua::lua_tolstring(L, 1).unwrap();
    let recursive = lua::lua_toboolean(L, 2);
    let mut files = Vec::new();
    visit_dirs(Path::new(path.as_str()), recursive, &mut files);
    return Luakit::variadic_return1(L, files);
}

pub fn lstdfs_mkdir(L: *mut lua_State) -> int {
    let path = lua::lua_tolstring(L, 1).unwrap();
    let res = fs::create_dir_all(path);
    match res {
        Ok(_) => Luakit::variadic_return1(L, true),
        Err(e) => Luakit::variadic_return2(L, false, e.to_string())
    }
}

pub fn lstdfs_remove(L: *mut lua_State) -> int {
    let path = lua::lua_tolstring(L, 1).unwrap();
    let rmall = lua::lua_toboolean(L, 2);
    let fpath = Path::new(path.as_str());
    let res = if fpath.is_dir() {
        if rmall { fs::remove_dir_all(path) } else { fs::remove_dir(path) }
    } else {
        fs::remove_file(path)
    };
    match res {
        Ok(_) => Luakit::variadic_return1(L, true),
        Err(e) => Luakit::variadic_return2(L, false, e.to_string())
    }
}

pub fn lstdfs_copy_file(L: *mut lua_State) -> int {
    let from = lua::lua_tolstring(L, 1).unwrap();
    let to = lua::lua_tolstring(L, 2).unwrap();
    let res = fs::copy(from, to);
    match res {
        Ok(_) => Luakit::variadic_return1(L, true),
        Err(e) => Luakit::variadic_return2(L, false, e.to_string())
    }
}

pub fn lstdfs_copy(L: *mut lua_State) -> int {
    let from = lua::lua_tolstring(L, 1).unwrap();
    let to = lua::lua_tolstring(L, 2).unwrap();
    let res = copy_dir_all(from, to);
    match res {
        Ok(_) => Luakit::variadic_return1(L, true),
        Err(e) => Luakit::variadic_return2(L, false, e.to_string())
    }
}

pub fn lstdfs_rename(L: *mut lua_State) -> int {
    let pold = lua::lua_tolstring(L, 1).unwrap();
    let pnew = lua::lua_tolstring(L, 2).unwrap();
    let res = fs::rename(pold, pnew);
    match res {
        Ok(_) => Luakit::variadic_return1(L, true),
        Err(e) => Luakit::variadic_return2(L, false, e.to_string())
    }
}

pub fn lstdfs_stem(L: *mut lua_State) -> int {
    let path = lua::lua_tolstring(L, 1).unwrap();
    let fpath = Path::new(path.as_str());
    return Luakit::variadic_return1(L, fpath.file_stem().unwrap().to_str().unwrap());
}

pub fn lstdfs_filename(L: *mut lua_State) -> int {
    let path = lua::lua_tolstring(L, 1).unwrap();
    let fpath = Path::new(path.as_str());
    return Luakit::variadic_return1(L, fpath.file_name().unwrap().to_str().unwrap());
}

pub fn lstdfs_extension(L: *mut lua_State) -> int {
    let path = lua::lua_tolstring(L, 1).unwrap();
    let fpath = Path::new(path.as_str());
    return Luakit::variadic_return1(L, fpath.extension().unwrap().to_str().unwrap());
}

pub fn lstdfs_parent_path(L: *mut lua_State) -> int {
    let path = lua::lua_tolstring(L, 1).unwrap();
    let fpath = Path::new(path.as_str());
    return Luakit::variadic_return1(L, fpath.parent().unwrap().to_str().unwrap());
}

pub fn lstdfs_append(L: *mut lua_State) -> int {
    let path = lua::lua_tolstring(L, 1).unwrap();
    let append = lua::lua_tolstring(L, 2).unwrap();
    let fpath = Path::new(path.as_str());
    return Luakit::variadic_return1(L, fpath.join(append).to_str().unwrap());
}

pub fn lstdfs_concat(L: *mut lua_State) -> int {
    let mut path = lua::lua_tolstring(L, 1).unwrap();
    let append = lua::lua_tolstring(L, 2).unwrap();
    path.push_str(append.as_str());
    return Luakit::variadic_return1(L, path);
}

pub fn lstdfs_exists(L: *mut lua_State) -> int {
    let path = lua::lua_tolstring(L, 1).unwrap();
    let fpath = Path::new(path.as_str());
    return Luakit::variadic_return1(L, fpath.exists());
}

pub fn lstdfs_is_directory(L: *mut lua_State) -> int {
    let path = lua::lua_tolstring(L, 1).unwrap();
    let fpath = Path::new(path.as_str());
    return Luakit::variadic_return1(L, fpath.is_dir());
}

pub fn lstdfs_is_absolute(L: *mut lua_State) -> int {
    let path = lua::lua_tolstring(L, 1).unwrap();
    let fpath = Path::new(path.as_str());
    return Luakit::variadic_return1(L, fpath.is_absolute());
}

pub fn lstdfs_split(L: *mut lua_State) -> int {
    let path = lua::lua_tolstring(L, 1).unwrap();
    let fpath = Path::new(path.as_str());
    let mut values = Vec::new();
    for component in fpath.components() {
        values.push(component.as_os_str().to_str().unwrap());
    }
    return Luakit::variadic_return1(L, values);
}

pub fn lstdfs_filetype(L: *mut lua_State) -> int {
    let path = lua::lua_tolstring(L, 1).unwrap();
    let meta = fs::metadata(path);
    return Luakit::variadic_return1(L, get_file_type(&meta));
}

pub fn lstdfs_file_size(L: *mut lua_State) -> int {
    let path = lua::lua_tolstring(L, 1).unwrap();
    let meta = fs::metadata(path);
    return Luakit::variadic_return1(L, metadata_find!(meta, file_size, 0));
}

pub fn lstdfs_last_write_time(L: *mut lua_State) -> int {
    let path = lua::lua_tolstring(L, 1).unwrap();
    let meta = fs::metadata(path);
    return Luakit::variadic_return1(L, metadata_find!(meta, last_write_time, 0));
}

pub fn lstdfs_current_path(L: *mut lua_State) -> int {
    let res = env::current_dir();
    match res {
        Ok(path) => Luakit::variadic_return1(L, path.to_str().unwrap()),
        Err(e) => Luakit::variadic_return2(L, lua::LUA_NIL, e.to_string())
    }
}

pub fn lstdfs_chdir(L: *mut lua_State) -> int {
    let path = lua::lua_tolstring(L, 1).unwrap();
    let res = env::set_current_dir(path);
    match res {
        Ok(_) => Luakit::variadic_return1(L, true),
        Err(e) => Luakit::variadic_return2(L, false, e.to_string())
    }
}

pub fn lstdfs_temp_dir(L: *mut lua_State) -> int {
    let path = env::temp_dir();
    return Luakit::variadic_return1(L, path.to_str().unwrap());
}

pub fn lstdfs_absolute(L: *mut lua_State) -> int {
    let path = lua::lua_tolstring(L, 1).unwrap();
    let res = path::absolute(path.clone());
    match res {
        Ok(abs) => Luakit::variadic_return1(L, abs.to_str().unwrap()),
        Err(_) => Luakit::variadic_return1(L, path)
    }
}

pub fn lstdfs_relative(L: *mut lua_State) -> int {
    let path = lua::lua_tolstring(L, 1).unwrap();
    let base = lua::lua_tolstring(L, 2).unwrap();
    match get_relative_path(base, path) {
        Some(p) => Luakit::variadic_return1(L, p.to_str().unwrap()),
        None => Luakit::variadic_return1(L, "")
    }
}

pub fn lstdfs_make_preferred(L: *mut lua_State) -> int {
    let path = lua::lua_tolstring(L, 1).unwrap();
    let fpath = Path::new(path.as_str());
    #[cfg(windows)]
    {
        Luakit::variadic_return1(L, fpath.to_string_lossy().replace("/", "\\"))
    }
    #[cfg(not(windows))]
    {
        Luakit::variadic_return1(L, fpath.to_string_lossy().replace("/", "\\"))
    }
}

pub fn lstdfs_root_name(L: *mut lua_State) -> int {
    let path = lua::lua_tolstring(L, 1).unwrap();
    let fpath = Path::new(path.as_str());
    if fpath.has_root() {
        let mut result = PathBuf::new();
        for component in fpath.components() {
            match component {
                Component::Prefix(prefix) => result.push(prefix.as_os_str()),
                _ => break,
            }
        }
        Luakit::variadic_return1(L, result.to_str().unwrap())
    } else {
        Luakit::variadic_return1(L, "".to_string())
    }
}

pub fn lstdfs_root_path(L: *mut lua_State) -> int {
    let path = lua::lua_tolstring(L, 1).unwrap();
    let fpath = Path::new(path.as_str());
    if fpath.has_root() {
        let mut result = PathBuf::new();
        for component in fpath.components() {
            match component {
                Component::RootDir => result.push(component.as_os_str()),
                Component::Prefix(prefix) => result.push(prefix.as_os_str()),
                _ => break,
            }
        }
        Luakit::variadic_return1(L, result.to_str().unwrap())
    } else {
        Luakit::variadic_return1(L, "".to_string())
    }
}

pub fn lstdfs_relative_path(L: *mut lua_State) -> int {
    let path = lua::lua_tolstring(L, 1).unwrap();
    let fpath = Path::new(path.as_str());
    if fpath.has_root() {
        let mut result = PathBuf::new();
        for component in fpath.components() {
            match component {
                Component::RootDir | Component::Prefix(_) => continue,
                _ => result.push(component.as_os_str()),
            }
        }
        Luakit::variadic_return1(L, result.to_str().unwrap())
    } else {
        Luakit::variadic_return1(L, fpath.to_path_buf().to_str().unwrap())
    }
}

pub fn lstdfs_replace_extension(L: *mut lua_State) -> int {
    let path = lua::lua_tolstring(L, 1).unwrap();
    let ext = lua::lua_tolstring(L, 2).unwrap();
    let fpath = Path::new(path.as_str());
    Luakit::variadic_return1(L, fpath.with_extension(ext).to_str().unwrap())
}

pub fn lstdfs_replace_filename(L: *mut lua_State) -> int {
    let path = lua::lua_tolstring(L, 1).unwrap();
    let fname = lua::lua_tolstring(L, 2).unwrap();
    let fpath = Path::new(path.as_str());
    Luakit::variadic_return1(L, fpath.with_file_name(fname).to_str().unwrap())
}

pub fn lstdfs_remove_filename(L: *mut lua_State) -> int {
    let path = lua::lua_tolstring(L, 1).unwrap();
    let fpath = Path::new(path.as_str());
    if fpath.as_os_str().to_str().map_or(false, |s| s.ends_with(MAIN_SEPARATOR)) {
        return Luakit::variadic_return1(L, fpath.to_path_buf().to_str().unwrap());
    }
    match fpath.parent() {
        Some(parent) => Luakit::variadic_return1(L, parent.to_path_buf().to_str().unwrap()),
        None => Luakit::variadic_return1(L, "".to_string()),
    }
}
