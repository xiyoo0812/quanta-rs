#![allow(non_snake_case)]
#![allow(dead_code)]

use std::fs::Metadata;
use std::{io, env, fs};
use std::time::UNIX_EPOCH;
use std::path::MAIN_SEPARATOR;
use std::path::{ self, Path, PathBuf, Component };

use libc::c_int as int;
use lua::{ cstr, lua_State };

use luakit::LuaPush;

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
    let path = lua::to_utf8(lua::lua_tolstring(L, 1));
    let recursive = lua::lua_toboolean(L, 2);
    let mut files = Vec::new();
    visit_dirs(Path::new(path.as_str()), recursive, &mut files);
    return luakit::vector_return!(L, files);
}

pub fn lstdfs_mkdir(L: *mut lua_State) -> int {
    let path = lua::to_utf8(lua::lua_tolstring(L, 1));
    let res = fs::create_dir_all(path);
    match res {
        Ok(_) => true.native_to_lua(L),
        Err(e) => luakit::variadic_return!(L, false, e.to_string())
    }
}

pub fn lstdfs_remove_file(L: *mut lua_State) -> int {
    let path = lua::to_utf8(lua::lua_tolstring(L, 1));
    match fs::remove_file(path) {
        Ok(_) => true.native_to_lua(L),
        Err(e) => luakit::variadic_return!(L, false, e.to_string())
    }
}

pub fn lstdfs_remove(L: *mut lua_State) -> int {
    let path = lua::to_utf8(lua::lua_tolstring(L, 1));
    let rmall = lua::lua_toboolean(L, 2);
    let res = if rmall { fs::remove_dir_all(path) } else { fs::remove_dir(path) };
    match res {
        Ok(_) => true.native_to_lua(L),
        Err(e) => luakit::variadic_return!(L, false, e.to_string())
    }
}

pub fn lstdfs_copy_file(L: *mut lua_State) -> int {
    let from = lua::to_utf8(lua::lua_tolstring(L, 1));
    let to = lua::to_utf8(lua::lua_tolstring(L, 2));
    let res = fs::copy(from, to);
    match res {
        Ok(_) => true.native_to_lua(L),
        Err(e) => luakit::variadic_return!(L, false, e.to_string())
    }
}

pub fn lstdfs_copy(L: *mut lua_State) -> int {
    let from = lua::to_utf8(lua::lua_tolstring(L, 1));
    let to = lua::to_utf8(lua::lua_tolstring(L, 2));
    let res = copy_dir_all(from, to);
    match res {
        Ok(_) => true.native_to_lua(L),
        Err(e) => luakit::variadic_return!(L, false, e.to_string())
    }
}

pub fn lstdfs_rename(L: *mut lua_State) -> int {
    let pold = lua::to_utf8(lua::lua_tolstring(L, 1));
    let pnew = lua::to_utf8(lua::lua_tolstring(L, 2));
    let res = fs::rename(pold, pnew);
    match res {
        Ok(_) => true.native_to_lua(L),
        Err(e) => luakit::variadic_return!(L, false, e.to_string())
    }
}

pub fn lstdfs_stem(L: *mut lua_State) -> int {
    let path = lua::to_utf8(lua::lua_tolstring(L, 1));
    let fpath = Path::new(path.as_str());
    fpath.file_stem().unwrap().to_str().unwrap().native_to_lua(L)
}

pub fn lstdfs_filename(L: *mut lua_State) -> int {
    let path = lua::to_utf8(lua::lua_tolstring(L, 1));
    let fpath = Path::new(path.as_str());
    fpath.file_name().unwrap().to_str().unwrap().native_to_lua(L)
}

pub fn lstdfs_extension(L: *mut lua_State) -> int {
    let path = lua::to_utf8(lua::lua_tolstring(L, 1));
    let fpath = Path::new(path.as_str());
    let ext = format!(".{}", fpath.extension().unwrap().to_str().unwrap());
    ext.native_to_lua(L)
}

pub fn lstdfs_parent_path(L: *mut lua_State) -> int {
    let path = lua::to_utf8(lua::lua_tolstring(L, 1));
    let fpath = Path::new(path.as_str());
    fpath.file_name().unwrap().to_str().unwrap().native_to_lua(L)
}

pub fn lstdfs_append(L: *mut lua_State) -> int {
    let path = lua::to_utf8(lua::lua_tolstring(L, 1));
    let append = lua::to_utf8(lua::lua_tolstring(L, 2));
    let fpath = Path::new(path.as_str());
    fpath.join(append).to_str().unwrap().native_to_lua(L)
}

pub fn lstdfs_concat(L: *mut lua_State) -> int {
    let mut path = lua::to_utf8(lua::lua_tolstring(L, 1));
    let append = lua::to_utf8(lua::lua_tolstring(L, 2));
    path.push_str(append.as_str());
    path.native_to_lua(L)
}

pub fn lstdfs_exists(L: *mut lua_State) -> int {
    let path = lua::to_utf8(lua::lua_tolstring(L, 1));
    let fpath = Path::new(path.as_str());
    fpath.exists().native_to_lua(L)
}

pub fn lstdfs_is_directory(L: *mut lua_State) -> int {
    let path = lua::to_utf8(lua::lua_tolstring(L, 1));
    let fpath = Path::new(path.as_str());
    fpath.is_dir().native_to_lua(L)
}

pub fn lstdfs_is_absolute(L: *mut lua_State) -> int {
    let path = lua::to_utf8(lua::lua_tolstring(L, 1));
    let fpath = Path::new(path.as_str());
    fpath.is_absolute().native_to_lua(L)
}

pub fn lstdfs_split(L: *mut lua_State) -> int {
    let path = lua::to_utf8(lua::lua_tolstring(L, 1));
    let fpath = Path::new(path.as_str());
    let mut values = Vec::new();
    for component in fpath.components() {
        values.push(component.as_os_str().to_str().unwrap());
    }
    return luakit::vector_return!(L, values);
}

pub fn lstdfs_filetype(L: *mut lua_State) -> int {
    let path = lua::to_utf8(lua::lua_tolstring(L, 1));
    let meta = fs::metadata(path);
    get_file_type(&meta).native_to_lua(L)
}

pub fn lstdfs_file_size(L: *mut lua_State) -> int {
    let path = lua::to_utf8(lua::lua_tolstring(L, 1));
    let rmeta = fs::metadata(path);
    rmeta.map(|m| m.len()).unwrap_or(0).native_to_lua(L)
}

pub fn lstdfs_last_write_time(L: *mut lua_State) -> int {
    let path = lua::to_utf8(lua::lua_tolstring(L, 1));
    let rmeta = fs::metadata(path);
    let stime = rmeta.map(|m| m.modified().unwrap_or(UNIX_EPOCH));
    let mtime = stime.map(|t| t.duration_since(UNIX_EPOCH).unwrap().as_secs()).unwrap_or(0);
    mtime.native_to_lua(L)
}

pub fn lstdfs_current_path(L: *mut lua_State) -> int {
    let res = env::current_dir();
    match res {
        Ok(path) => path.to_str().unwrap().native_to_lua(L),
        Err(e) => luakit::variadic_return!(L, lua::LUA_NIL, e.to_string())
    }
}

pub fn lstdfs_chdir(L: *mut lua_State) -> int {
    let path = lua::to_utf8(lua::lua_tolstring(L, 1));
    let res = env::set_current_dir(path);
    match res {
        Ok(_) => true.native_to_lua(L),
        Err(e) => luakit::variadic_return!(L, false, e.to_string())
    }
}

pub fn lstdfs_temp_dir(L: *mut lua_State) -> int {
    env::temp_dir().to_str().unwrap().native_to_lua(L)
}

pub fn lstdfs_absolute(L: *mut lua_State) -> int {
    let path = lua::to_utf8(lua::lua_tolstring(L, 1));
    let res = path::absolute(path.clone());
    match res {
        Ok(abs) => abs.to_str().unwrap().native_to_lua(L),
        Err(_) => path.native_to_lua(L)
    }
}

pub fn lstdfs_relative(L: *mut lua_State) -> int {
    let path = lua::to_utf8(lua::lua_tolstring(L, 1));
    let base = lua::to_utf8(lua::lua_tolstring(L, 2));
    match get_relative_path(base, path) {
        Some(p) => p.to_str().unwrap().native_to_lua(L),
        None => "".native_to_lua(L)
    }
}

pub fn lstdfs_make_preferred(L: *mut lua_State) -> int {
    let path = lua::to_utf8(lua::lua_tolstring(L, 1));
    let fpath = Path::new(path.as_str());
    #[cfg(windows)]
    {
        fpath.to_string_lossy().replace("/", "\\").native_to_lua(L)
    }
    #[cfg(not(windows))]
    {
        fpath.to_string_lossy().replace("/", "\\").native_to_lua(L)
    }
}

pub fn lstdfs_root_name(L: *mut lua_State) -> int {
    let path = lua::to_utf8(lua::lua_tolstring(L, 1));
    let fpath = Path::new(path.as_str());
    if fpath.has_root() {
        let mut result = PathBuf::new();
        for component in fpath.components() {
            match component {
                Component::Prefix(prefix) => result.push(prefix.as_os_str()),
                _ => break,
            }
        }
        result.to_str().unwrap().native_to_lua(L)
    } else {
        "".native_to_lua(L)
    }
}

pub fn lstdfs_root_path(L: *mut lua_State) -> int {
    let path = lua::to_utf8(lua::lua_tolstring(L, 1));
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
        result.to_str().unwrap().native_to_lua(L)
    } else {
        "".native_to_lua(L)
    }
}

pub fn lstdfs_relative_path(L: *mut lua_State) -> int {
    let path = lua::to_utf8(lua::lua_tolstring(L, 1));
    let fpath = Path::new(path.as_str());
    if fpath.has_root() {
        let mut result = PathBuf::new();
        for component in fpath.components() {
            match component {
                Component::RootDir | Component::Prefix(_) => continue,
                _ => result.push(component.as_os_str()),
            }
        }
        result.to_str().unwrap().native_to_lua(L)
    } else {
        fpath.to_path_buf().to_str().unwrap().native_to_lua(L)
    }
}

pub fn lstdfs_replace_extension(L: *mut lua_State) -> int {
    let path = lua::to_utf8(lua::lua_tolstring(L, 1));
    let ext = lua::to_utf8(lua::lua_tolstring(L, 2));
    let fpath = Path::new(path.as_str());
    fpath.with_extension(ext).to_str().unwrap().native_to_lua(L)
}

pub fn lstdfs_replace_filename(L: *mut lua_State) -> int {
    let path = lua::to_utf8(lua::lua_tolstring(L, 1));
    let fname = lua::to_utf8(lua::lua_tolstring(L, 2));
    let fpath = Path::new(path.as_str());
    fpath.with_file_name(fname).to_str().unwrap().native_to_lua(L)
}

pub fn lstdfs_remove_filename(L: *mut lua_State) -> int {
    let path = lua::to_utf8(lua::lua_tolstring(L, 1));
    let fpath = Path::new(path.as_str());
    if fpath.as_os_str().to_str().map_or(false, |s| s.ends_with(MAIN_SEPARATOR)) {
        return fpath.to_path_buf().to_str().unwrap().native_to_lua(L);
    }
    match fpath.parent() {
        Some(parent) => parent.to_path_buf().to_str().unwrap().native_to_lua(L),
        None => "".native_to_lua(L),
    }
}
