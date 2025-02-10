#![allow(non_snake_case)]
#![allow(static_mut_refs)]

extern crate lua;
extern crate libc;
extern crate chrono;
extern crate luakit;
extern crate dyn_fmt;

mod logger;

use libc::c_int as int;
use libc::c_char as char;

use std::sync::Arc;
use std::sync::Mutex;
use dyn_fmt::Arguments;
use once_cell::sync::Lazy;
use lua::{ from_cstr, ternary, lua_State };
use logger::{ LogLevel, RollingType, LogService };
use luakit::{ Luakit, LuaRead, LuaPush, LuaPushFn };

const LOG_FLAG_FORMAT: i32 = 1;
const LOG_FLAG_PRETTY: i32 = 2;
const LOG_FLAG_MONITOR: i32 = 4;

static S_LOGGER: Lazy<Arc<Mutex<LogService>>> = Lazy::new(|| {
    Arc::new(Mutex::new(LogService::new()))
});

fn read_args(L: *mut lua_State, flag: int, index: int) -> String {
    let t = unsafe { lua::lua_type(L, index) };
    match t {
        lua::LUA_TNIL => "nil".to_string(),
        lua::LUA_TTHREAD => "thread".to_string(),
        lua::LUA_TFUNCTION => "function".to_string(),
        lua::LUA_TUSERDATA | lua::LUA_TLIGHTUSERDATA => "userdata".to_string(),
        lua::LUA_TBOOLEAN => ternary!(lua::lua_toboolean(L, index), "true".to_string(), "false".to_string()),
        lua::LUA_TSTRING => lua::lua_tolstring(L, index).unwrap(),
        lua::LUA_TNUMBER => {
            if unsafe { lua::lua_isinteger(L, index) } == 1 {
                return format!("{}", lua::lua_tointeger(L, index));
            }
            format!("{}", lua::lua_tonumber(L, index))
        }
        lua::LUA_TTABLE => {
            if (flag & LOG_FLAG_FORMAT) == LOG_FLAG_FORMAT {
                return lua::lua_tolstring(L, index).unwrap();
            }
            lua::lua_tolstring(L, index).unwrap()
        } 
        _ => "unsuppert data type".to_string(),
    }
}

#[no_mangle]
pub extern "C" fn luaopen_lualog(L: *mut lua_State) -> int {
    let mut kit = Luakit::load(L);
    let mut lualog = kit.new_table(Some("log"));
    luakit::new_enum!(lualog, "LOG_LEVEL",
        "INFO", LogLevel::INFO,
        "WARN", LogLevel::WARN,
        "DUMP", LogLevel::DUMP,
        "DEBUG", LogLevel::DEBUG,
        "ERROR", LogLevel::ERROR,
        "FATAL", LogLevel::FATAL
    );
    luakit::new_enum!(lualog, "LOG_FLAG",
        "NULL", 0,
        "FORMAT", LOG_FLAG_FORMAT,
        "PRETTY", LOG_FLAG_PRETTY,
        "MONITOR", LOG_FLAG_MONITOR
    );
    luakit::new_enum!(lualog, "ROLLING_TYPE",
        "DAYLY", RollingType::DAYLY,
        "HOURLY", RollingType::HOURLY
    );
    luakit::set_function!(lualog, "daemon", |status : bool | {
        S_LOGGER.lock().unwrap().daemon(status);
    });
    luakit::set_function!(lualog, "set_max_size", |size : usize | {
        S_LOGGER.lock().unwrap().set_max_size(size);
    });
    luakit::set_function!(lualog, "set_clean_time", |time : u64 | {
        S_LOGGER.lock().unwrap().set_clean_time(time);
    });
    luakit::set_function!(lualog, "filter", |lv : u32, on: bool | {
        S_LOGGER.lock().unwrap().filter(lv, on);
    });
    luakit::set_function!(lualog, "is_filter", |lv : u32 | {
        S_LOGGER.lock().unwrap().is_filter(lv);
    });
    luakit::set_function!(lualog, "set_rolling_type", |tp : u32 | {
        S_LOGGER.lock().unwrap().set_rolling_type(tp);
    });
    luakit::set_function!(lualog, "add_dest", |feature : String | {
        S_LOGGER.lock().unwrap().add_dest(feature);
    });
    luakit::set_function!(lualog, "del_dest", |feature : String | {
        S_LOGGER.lock().unwrap().del_dest(feature);
    });
    luakit::set_function!(lualog, "add_file_dest", |feature : String, fname : String | {
        S_LOGGER.lock().unwrap().add_file_dest(feature, fname);
    });
    luakit::set_function!(lualog, "set_dest_clean_time", |feature : String, time : u64 | {
        S_LOGGER.lock().unwrap().set_dest_clean_time(feature, time);
    });
    luakit::set_function!(lualog, "ignore_prefix", |feature : String, prefix : bool | {
        S_LOGGER.lock().unwrap().ignore_prefix(feature, prefix);
    });
    luakit::set_function!(lualog, "ignore_suffix", |feature : String, suffix : bool | {
        S_LOGGER.lock().unwrap().ignore_suffix(feature, suffix);
    });    
    luakit::set_function!(lualog, "add_lvl_dest", |lv : u32 | {
        S_LOGGER.lock().unwrap().add_lvl_dest(lv);
    });    
    luakit::set_function!(lualog, "del_lvl_dest", |lv : u32 | {
        S_LOGGER.lock().unwrap().del_lvl_dest(lv);
    });
    luakit::set_function!(lualog, "option", |path: String, service: String, index: String | {
        let logger = Arc::clone(&S_LOGGER);
        S_LOGGER.lock().unwrap().option(logger, path, service, index);
    });
    lualog.set_function( "print", | L: *mut lua_State | -> int {
        let level: u32 = LuaRead::lua_to_native(L, 1).unwrap();
        if S_LOGGER.lock().unwrap().is_filter(level) {
            return 0;
        }
        let flag: i32 = LuaRead::lua_to_native(L, 2).unwrap();
        let tag: String = LuaRead::lua_to_native(L, 3).unwrap();
        let feature: String = LuaRead::lua_to_native(L, 4).unwrap();
        let vfmt: String = LuaRead::lua_to_native(L, 5).unwrap();
        let argc = unsafe { lua::lua_gettop(L) - 5};
        let mut luaargs: Vec<String> = Vec::new();
        for i in 0..argc {
            luaargs.push(read_args(L, flag, 6 + i));
        }
        let msg = Arguments::new(vfmt, &luaargs).to_string();
        S_LOGGER.lock().unwrap().output(level, &msg, tag, feature, "", 0);
        if (flag & LOG_FLAG_MONITOR) == LOG_FLAG_MONITOR {
            return msg.native_to_lua(L);
        }
        0
    });
    lualog.set_function( "format", | L: *mut lua_State | -> int {
        let vfmt: String = LuaRead::lua_to_native(L, 1).unwrap();
        let argc = unsafe { lua::lua_gettop(L) - 1};
        let mut luaargs: Vec<String> = Vec::new();
        for i in 0..argc {
            luaargs.push(read_args(L, LOG_FLAG_FORMAT, i + 2));
        }
        let msg = Arguments::new(vfmt, &luaargs).to_string();
        msg.native_to_lua(L)
    });
    lualog.native_to_lua(L)
}

#[no_mangle]
pub unsafe extern "C" fn init_logger() {
    S_LOGGER.lock().unwrap().start();
}

#[no_mangle]
pub unsafe extern "C" fn stop_logger() {
    S_LOGGER.lock().unwrap().stop();
}

#[no_mangle]
pub unsafe extern "C" fn option_logger(path: *const char, service: *const char, index: *const char) {
    let logger = Arc::clone(&S_LOGGER);
    S_LOGGER.lock().unwrap().option(logger, from_cstr(path).unwrap(), from_cstr(service).unwrap(), from_cstr(index).unwrap());
}

macro_rules! LOG_OUTPUT {
    ($name:ident, $level:expr, $tag:expr, $feature:expr, $source:expr, $line:expr) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(msg: *const char) {
            S_LOGGER.lock().unwrap().output($level as u32, &from_cstr(msg).unwrap(), $tag.to_string(), $feature.to_string(), $source, $line);
        }
    };
}
LOG_OUTPUT!(LOG_INFO, LogLevel::INFO, "", "", "", 0);
LOG_OUTPUT!(LOG_WARN, LogLevel::WARN, "", "", "", 0);
LOG_OUTPUT!(LOG_DUMP, LogLevel::DUMP, "", "", "", 0);
LOG_OUTPUT!(LOG_FATAL, LogLevel::FATAL, "", "", "", 0);
LOG_OUTPUT!(LOG_DEBUG, LogLevel::DEBUG, "", "", "", 0);
LOG_OUTPUT!(LOG_ERROR, LogLevel::ERROR, "", "", "", 0);