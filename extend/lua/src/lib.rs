#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(improper_ctypes)]

extern crate libc;

use libc::c_int as int;
use libc::c_void as void;
use libc::c_char as char;
use libc::c_uchar as uchar;
use libc::size_t as size_t;
use std::{ default, ptr };

pub const MULTRET: int              = -1;

pub const LUA_OK: int               = 0;
pub const LUA_YIELD: int            = 1;
pub const LUA_ERRRUN: int           = 2;
pub const LUA_ERRSYNTAX: int        = 3;
pub const LUA_ERRMEM: int           = 4;
pub const LUA_ERRGCMM: int          = 5;
pub const LUA_ERRERR: int           = 6;

pub const LUA_TNONE: int            = -1;
pub const LUA_TNIL: int             = 0;
pub const LUA_TBOOLEAN: int         = 1;
pub const LUA_TLIGHTUSERDATA: int   = 2;
pub const LUA_TNUMBER: int          = 3;
pub const LUA_TSTRING: int          = 4;
pub const LUA_TTABLE: int           = 5;
pub const LUA_TFUNCTION: int        = 6;
pub const LUA_TUSERDATA: int        = 7;
pub const LUA_TTHREAD: int          = 8;

pub const LUA_MINSTACK: int         = 20;

pub const LUA_RIDX_MAINTHREAD: int  = 1;
pub const LUA_RIDX_GLOBALS: int     = 2;

pub const LUA_OPADD: int            = 0;
pub const LUA_OPSUB: int            = 1;
pub const LUA_OPMUL: int            = 2;
pub const LUA_OPDIV: int            = 3;
pub const LUA_OPMOD: int            = 4;
pub const LUA_OPPOW: int            = 5;
pub const LUA_OPUNM: int            = 6;

pub const LUA_OPEQ: int             = 0;
pub const LUA_OPLT: int             = 1;
pub const LUA_OPLE: int             = 2;

pub const LUA_GCSTOP: int           = 0;
pub const LUA_GCRESTART: int        = 1;
pub const LUA_GCCOLLECT: int        = 2;
pub const LUA_GCCOUNT: int          = 3;
pub const LUA_GCCOUNTB: int         = 4;
pub const LUA_GCSTEP: int           = 5;
pub const LUA_GCSETPAUSE: int       = 6;
pub const LUA_GCSETSTEPMUL: int     = 7;
pub const LUA_GCSETMAJORINC: int    = 8;
pub const LUA_GCISRUNNING: int      = 9;
pub const LUA_GCGEN: int            = 10;
pub const LUA_GCINC: int            = 11;

pub const LUA_HOOKCALL: int         = 0;
pub const LUA_HOOKRET: int          = 1;
pub const LUA_HOOKLINE: int         = 2;
pub const LUA_HOOKCOUNT: int        = 3;
pub const LUA_HOOKTAILRET: int      = 4;

pub const LUA_MASKCALL: int         = 1 << LUA_HOOKCALL as usize;
pub const LUA_MASKRET: int          = 1 << LUA_HOOKRET as usize;
pub const LUA_MASKLINE: int         = 1 << LUA_HOOKLINE as usize;
pub const LUA_MASKCOUNT: int        = 1 << LUA_HOOKCOUNT as usize;

pub const LUAI_MAXSTACK: int        = 1000000;  // or 15000 with 32b    // TODO:
pub const LUAI_FIRSTPSEUDOIDX: int  = -LUAI_MAXSTACK - 1000;
pub const LUA_REGISTRYINDEX: int    = LUAI_FIRSTPSEUDOIDX;

pub type lua_Number                 = libc::c_double;
pub type lua_Integer                = libc::ptrdiff_t;
pub type lua_Unsigned               = libc::c_ulong;

#[repr(C)]
pub struct lua_State;
pub type lua_CFunction  = extern "C" fn(L: *mut lua_State) -> int;
pub type lua_Hook       = extern "C" fn(L: *mut lua_State, ar: *mut lua_Debug);
pub type lua_Reader     = extern "C" fn(L: *mut lua_State, ud: *mut void, sz: *mut size_t) -> *const char;
pub type lua_Writer     = extern "C" fn(L: *mut lua_State, p: *const void, sz: size_t, ud: *mut void) -> int;
pub type lua_Alloc      = extern "C" fn(ud: *mut void, ptr: *mut void, osize: size_t, nsize: size_t) -> *mut void;

#[repr(C)]
#[allow(missing_copy_implementations)]
pub struct lua_Debug {
    pub event: int,
    pub name: *const char,
    pub namewhat: *const char,
    pub what: *const char,
    pub source: *const char,
    pub currentline: int,
    pub linedefined: int,
    pub lastlinedefined: int,
    pub nups: uchar,
    pub nparams: uchar,
    pub isvararg: char,
    pub istailcall: char,
    pub short_src: [char; 60], // i_ci: *CallInfo
}

extern "C" {
    pub fn lua_close(L: *mut lua_State);
    pub fn lua_newthread(L: *mut lua_State) -> *mut lua_State;
    pub fn lua_newstate(f: lua_Alloc, ud: *mut void) -> *mut lua_State;

    pub fn lua_atpanic(L: *mut lua_State, panicf: lua_CFunction) -> lua_CFunction;

    pub fn lua_version(L: *mut lua_State) -> *const lua_Number;

    pub fn lua_absindex(L: *mut lua_State, idx: int) -> int;
    pub fn lua_gettop(L: *mut lua_State) -> int;
    pub fn lua_settop(L: *mut lua_State, idx: int);
    pub fn lua_pushvalue(L: *mut lua_State, idx: int);
    pub fn lua_rotate(L: *mut lua_State, idx: int, n: int);

    pub fn lua_replace(L: *mut lua_State, idx: int);
    pub fn lua_copy(L: *mut lua_State, fromidx: int, toidx: int);
    pub fn lua_checkstack(L: *mut lua_State, sz: int) -> int;

    pub fn lua_xmove(from: *mut lua_State, to: *mut lua_State, n: int);

    pub fn lua_isnumber(L: *mut lua_State, idx: int) -> int;
    pub fn lua_isstring(L: *mut lua_State, idx: int) -> int;
    pub fn lua_iscfunction(L: *mut lua_State, idx: int) -> int;
    pub fn lua_isuserdata(L: *mut lua_State, idx: int) -> int;
    pub fn lua_type(L: *mut lua_State, idx: int) -> int;
    pub fn lua_typename(L: *mut lua_State, tp: int) -> *const char;

    pub fn lua_arith(L: *mut lua_State, op: int);
    pub fn lua_rawlen(L: *mut lua_State, idx: int) -> size_t;
    pub fn lua_rawequal(L: *mut lua_State, idx1: int, idx2: int) -> int;
    pub fn lua_compare(L: *mut lua_State, idx1: int, idx2: int, op: int) -> int;

    pub fn lua_toboolean(L: *mut lua_State, idx: int) -> int;
    pub fn lua_tonumberx(L: *mut lua_State, idx: int, isnum: *mut int) -> lua_Number;
    pub fn lua_tointegerx(L: *mut lua_State, idx: int, isnum: *mut int) -> lua_Integer;
    pub fn lua_tolstring(L: *mut lua_State, idx: int, len: *mut size_t) -> *const char;
    pub fn lua_tocfunction(L: *mut lua_State, idx: int) -> Option<lua_CFunction>;
    pub fn lua_touserdata(L: *mut lua_State, idx: int) -> *mut void;
    pub fn lua_topointer(L: *mut lua_State, idx: int) -> *const void;
    pub fn lua_tothread(L: *mut lua_State, idx: int) -> *mut lua_State;

    pub fn lua_pushnil(L: *mut lua_State);
    pub fn lua_pushnumber(L: *mut lua_State, n: lua_Number);
    pub fn lua_pushinteger(L: *mut lua_State, n: lua_Integer);
    pub fn lua_pushlstring(L: *mut lua_State, s: *const char, l: size_t);
    pub fn lua_pushstring(L: *mut lua_State, s: *const char);
    // TODO: lua_pushvfstring()
    pub fn lua_pushfstring(L: *mut lua_State, fmt: *const char, ...) -> *const char;
    pub fn lua_pushcclosure(L: *mut lua_State, f: lua_CFunction, n: int);
    pub fn lua_pushboolean(L: *mut lua_State, b: int);
    pub fn lua_pushlightuserdata(L: *mut lua_State, p: *mut void);
    pub fn lua_pushthread(L: *mut lua_State) -> int;

    pub fn lua_getglobal(L: *mut lua_State, var: *const char);
    pub fn lua_gettable(L: *mut lua_State, idx: int);
    pub fn lua_getfield(L: *mut lua_State, idx: int, k: *const char);
    pub fn lua_rawget(L: *mut lua_State, idx: int);
    pub fn lua_rawgeti(L: *mut lua_State, idx: int, n: int);
    pub fn lua_rawgetp(L: *mut lua_State, idx: int, p: *const char);
    pub fn lua_createtable(L: *mut lua_State, narr: int, nrec: int);
    pub fn lua_newuserdata(L: *mut lua_State, sz: size_t) -> *mut void;
    pub fn lua_getmetatable(L: *mut lua_State, objindex: int) -> int;
    pub fn lua_getfenv(L: *mut lua_State, idx: int);

    pub fn lua_setglobal(L: *mut lua_State, var: *const char);
    pub fn lua_settable(L: *mut lua_State, idx: int);
    pub fn lua_setfield(L: *mut lua_State, idx: int, k: *const char);
    pub fn lua_rawset(L: *mut lua_State, idx: int);
    pub fn lua_rawseti(L: *mut lua_State, idx: int, n: int);
    pub fn lua_rawsetp(L: *mut lua_State, idx: int, p: *const char);
    pub fn lua_setmetatable(L: *mut lua_State, objindex: int) -> int;
    pub fn lua_setfenv(L: *mut lua_State, idx: int) -> int;

    pub fn lua_getctx(L: *mut lua_State, ctx: int) -> int;
    pub fn lua_callk(L: *mut lua_State, nargs: int, nresults: int, ctx: int, k: Option<lua_CFunction>);
    pub fn lua_pcallk(L: *mut lua_State, nargs: int, nresults: int, errfunc: int, ctx: int, k: Option<lua_CFunction>) -> int;
    pub fn lua_load(L: *mut lua_State, reader: lua_Reader, dt: *mut void, chunkname: *const char, mode: *const char) -> int;
    pub fn lua_dump(L: *mut lua_State, writer: lua_Writer, data: *mut void) -> int;

    pub fn lua_yieldk(L: *mut lua_State, nresults: int, ctx: int, k: Option<lua_CFunction>) -> int;
    pub fn lua_resume(L: *mut lua_State, from: *mut lua_State, narg: int) -> int;
    pub fn lua_status(L: *mut lua_State) -> int;

    pub fn lua_gc(L: *mut lua_State, what: int, data: int) -> int;

    pub fn lua_error(L: *mut lua_State) -> int;
    pub fn lua_next(L: *mut lua_State, idx: int) -> int;
    pub fn lua_concat(L: *mut lua_State, n: int);
    pub fn lua_len(L: *mut lua_State, idx: int);

    pub fn lua_getallocf(L: *mut lua_State, ud: *mut *mut void) -> lua_Alloc;
    pub fn lua_setallocf(L: *mut lua_State, f: lua_Alloc, ud: *mut void);

    pub fn lua_getstack(L: *mut lua_State, level: int, ar: *mut lua_Debug) -> int;
    pub fn lua_getinfo(L: *mut lua_State, what: *const char, ar: *mut lua_Debug) -> int;
    pub fn lua_getlocal(L: *mut lua_State, ar: *const lua_Debug, n: int) -> *const char;
    pub fn lua_setlocal(L: *mut lua_State, ar: *mut lua_Debug, n: int) -> *const char;
    pub fn lua_getupvalue(L: *mut lua_State, funcindex: int, n: int) -> *const char;
    pub fn lua_setupvalue(L: *mut lua_State, funcindex: int, n: int) -> *const char;

    pub fn lua_upvalueid(L: *mut lua_State, fidx: int, n: int) -> *const void;
    pub fn lua_upvaluejoin(L: *mut lua_State, fidx1: int, n1: int, fidx2: int, n2: int);

    pub fn lua_sethook(L: *mut lua_State, func: lua_Hook, mask: int, count: int) -> int;
    pub fn lua_gethook(L: *mut lua_State) -> lua_Hook;
    pub fn lua_gethookmask(L: *mut lua_State) -> int;
    pub fn lua_gethookcount(L: *mut lua_State) -> int;

    pub fn luaL_openlibs(L: *mut lua_State);
    pub fn luaL_newstate() -> *mut lua_State;
    pub fn luaL_error(L: *mut lua_State, info: *const char);
    pub fn luaL_setmetatable(L: *mut lua_State, tname: *const char);
    pub fn luaL_loadstring(L: *mut lua_State, p: *const char) -> int;
    pub fn luaL_loadbufferx(L: *mut lua_State, buff: *const char, sz: size_t, name: *const char, mode: *const char) -> int;

}
#[inline(always)]
pub fn lua_upvalueindex(i: int) -> int {
    LUA_REGISTRYINDEX - i
}

#[inline(always)]
pub unsafe fn lua_call(L: *mut lua_State, nargs: int, nresults: int) {
    lua_callk(L, nargs, nresults, 0, None)
}

#[inline(always)]
pub unsafe fn lua_pcall(L: *mut lua_State, nargs: int, nresults: int, errfunc: int) -> int {
    lua_pcallk(L, nargs, nresults, errfunc, 0, None)
}

#[inline(always)]
pub unsafe fn lua_yield(L: *mut lua_State, nresults: int) -> int {
    lua_yieldk(L, nresults, 0, None)
}

#[inline(always)]
pub unsafe fn lua_pop(L: *mut lua_State, n: int) {
    lua_settop(L, -n - 1)
}

#[inline(always)]
pub unsafe fn lua_newtable(L: *mut lua_State) {
    lua_createtable(L, 0, 0)
}

#[inline(always)]
pub unsafe fn lua_register(L: *mut lua_State, name: *const char, f: lua_CFunction) {
    lua_pushcfunction(L, f);
    lua_setglobal(L, name)
}

#[inline(always)]
pub unsafe fn lua_pushcfunction(L: *mut lua_State, f: lua_CFunction) {
    lua_pushcclosure(L, f, 0)
}

#[inline(always)]
pub unsafe fn lua_isfunction(L: *mut lua_State, idx: int) -> bool {
    lua_type(L, idx) == LUA_TFUNCTION
}

#[inline(always)]
pub unsafe fn lua_istable(L: *mut lua_State, idx: int) -> bool {
    lua_type(L, idx) == LUA_TTABLE
}

#[inline(always)]
pub unsafe fn lua_islightuserdata(L: *mut lua_State, idx: int) -> bool {
    lua_type(L, idx) == LUA_TLIGHTUSERDATA
}

#[inline(always)]
pub unsafe fn lua_isnil(L: *mut lua_State, idx: int) -> bool {
    lua_type(L, idx) == LUA_TNIL
}

#[inline(always)]
pub unsafe fn lua_isboolean(L: *mut lua_State, idx: int) -> bool {
    lua_type(L, idx) == LUA_TBOOLEAN
}

#[inline(always)]
pub unsafe fn lua_isthread(L: *mut lua_State, idx: int) -> bool {
    lua_type(L, idx) == LUA_TTHREAD
}

#[inline(always)]
pub unsafe fn lua_isnone(L: *mut lua_State, idx: int) -> bool {
    lua_type(L, idx) == LUA_TNONE
}

#[inline(always)]
pub unsafe fn lua_isnoneornil(L: *mut lua_State, idx: int) -> bool {
    lua_type(L, idx) <= 0
}

#[inline(always)]
pub unsafe fn lua_pushglobaltable(L: *mut lua_State) {
    lua_rawgeti(L, LUA_REGISTRYINDEX, LUA_RIDX_GLOBALS)
}

pub unsafe fn lua_tostring(L: *mut lua_State, i: int) -> *const char {
    return lua_tolstring(L, i, ptr::null_mut());
}

pub unsafe fn lua_tonumber(L: *mut lua_State, i: int) -> lua_Number {
    return lua_tonumberx(L, i, ptr::null_mut());
}

pub unsafe fn lua_tointeger(L: *mut lua_State, i: int) -> lua_Integer {
    return lua_tointegerx(L, i, ptr::null_mut());
}

pub unsafe fn lua_tounsigned(L: *mut lua_State, i: int) -> lua_Unsigned {
    return lua_tointegerx(L, i, ptr::null_mut()) as lua_Unsigned;
}

pub unsafe fn lua_remove(L: *mut lua_State, idx: int) {
    lua_rotate(L, idx, -1);
    lua_pop(L, 1);
}

pub unsafe fn lua_insert(L: *mut lua_State, idx: int) {
    lua_rotate(L, idx, 1);
}

pub unsafe fn luaL_loadbuffer(L: *mut lua_State, buff: *const char, sz: size_t, name: *const char) -> int {
    luaL_loadbufferx(L, buff, sz, name, ptr::null_mut())
}

impl default::Default for lua_Debug {
    fn default() -> lua_Debug {
        lua_Debug {
            event: 0,
            name: ptr::null(),
            namewhat: ptr::null(),
            what: ptr::null(),
            source: ptr::null(),
            currentline: 0,
            linedefined: 0,
            lastlinedefined: 0,
            nups: 0,
            nparams: 0,
            isvararg: 0,
            istailcall: 0,
            short_src: [0; 60],
        }
    }
}
