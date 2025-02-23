#![allow(non_snake_case)]

extern crate lua;
extern crate libc;
extern crate luakit;

mod worker;
mod scheduler;

use lua::lua_State;
use libc::c_int as int;
use std::cell::RefCell;

use crate::worker::IScheduler;
use crate::scheduler::Scheduler;

use luakit::{ Luakit, LuaPush, LuaPushFn, LuaPushLuaFn, PtrWrapper };

thread_local! {
    static SCHEDULER: RefCell<PtrWrapper<Scheduler>> = RefCell::new(PtrWrapper::new(Box::new(Scheduler::new())));
}

#[no_mangle]
pub extern "C" fn luaopen_lworker(L: *mut lua_State) -> int {
    let mut kit = Luakit::load(L);
    let mut lworker = kit.new_table(Some("worker"));
    luakit::set_function!(lworker, "setup", |L: *mut lua_State, ns: String| {
        SCHEDULER.with_borrow(|scheduler| scheduler.clone().setup(L, ns))
    });
    luakit::set_function!(lworker, "shutdown", || {
        SCHEDULER.with_borrow(|scheduler| scheduler.clone().shutdown())
    });
    luakit::set_function!(lworker, "stop", |name: String| {
        SCHEDULER.with_borrow(|scheduler| scheduler.clone().stop(name))
    });
    luakit::set_function!(lworker, "update", |L: *mut lua_State, clock_ms: u64| {
        SCHEDULER.with_borrow(|scheduler| scheduler.clone().update(L, clock_ms))
    });
    luakit::set_function!(lworker, "broadcast", |L: *mut lua_State| {
        SCHEDULER.with_borrow(|scheduler| scheduler.clone().broadcast(L))
    });
    luakit::set_function!(lworker, "call", |L: *mut lua_State, name: String| {
        SCHEDULER.with_borrow(|scheduler| scheduler.clone().call(L, &name))
    });
    luakit::set_function!(lworker, "startup", |L: *mut lua_State, name: String, conf: String| {
        SCHEDULER.with_borrow(|scheduler| scheduler.clone().startup(L, name, conf))
    });
    lworker.native_to_lua(L)
}
