#![allow(non_snake_case)]
#![allow(dead_code)]

use lua::lua_State;

use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

pub fn hash_code(L: *mut lua_State) -> u64 {
    let ltype = unsafe { lua::lua_type(L, 1) };
    let mut value = match ltype {
        lua::LUA_TNUMBER => calculate_hash(&lua::lua_tointeger(L,1)),
        lua::LUA_TSTRING => calculate_hash(&lua::to_utf8(lua::lua_tolstring(L, 1))),
        _ => lua::luaL_error(L, "hashkey only support number or string!")
    };
    let modv = lua::luaL_optinteger(L, 2, 0);
    if modv > 0 {
        value %= modv;
    }
    value
}