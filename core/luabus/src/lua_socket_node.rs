#![allow(non_snake_case)]
#![allow(dead_code)]

use lua::lua_State;
use libc::c_int as int;

use std::rc::Weak;
use std::cell::RefCell;

use luakit::{LuaGc, Luakit};

use crate::socket_mgr::{ SocketMgr, Prototype };
use crate::socket_router::SocketRouter;

pub struct LuaSocketNode {
    sindex: u16,
    luavm: Luakit,
    ptype: Prototype,
    socket_mgr: Weak<RefCell<SocketMgr>>,
    socket_router: Weak<RefCell<SocketRouter>>,
    pub token: u32,
    pub stoken: u32,
    pub ip: String,
}

impl LuaGc for LuaSocketNode {}

impl LuaSocketNode {
    pub fn new(token: u32, L: *mut lua_State, mgr: Weak<RefCell<SocketMgr>>, router: Weak<RefCell<SocketRouter>>, ptype: Prototype) -> LuaSocketNode {
        LuaSocketNode {
            sindex : 1,
            ptype: ptype,
            token : token,
            socket_mgr: mgr,
            ip: "".to_string(),
            socket_router: router,
            luavm : Luakit::load(L),
            stoken : (token & 0xffff) << 16,
        }
    }

    pub fn close(&self) {
    }

    pub fn build_session_id(&mut self) -> u32 {
        self.sindex += 1;
        self.stoken | self.sindex as u32
    }

    pub fn get_route_count(&self) -> u32 {
        if let Some(router) = self.socket_router.upgrade() {
            return router.borrow_mut().get_route_count();
        }
        0
    }
    pub fn set_timeout(&self, ms: u64) {
        if let Some(socket_mgr) = self.socket_mgr.upgrade() {
            return socket_mgr.borrow_mut().set_timeout(self.token, ms);
        }
    }

    pub fn set_nodelay(&self, enable: bool) {
        if let Some(socket_mgr) = self.socket_mgr.upgrade() {
            return socket_mgr.borrow_mut().set_nodelay(self.token, enable);
        }
    }

    pub fn call_pb(&mut self, L: *mut lua_State) -> int{
        unsafe { lua::lua_pushinteger(L, 0) };
        1
    }
    pub fn call_data(&mut self, L: *mut lua_State) -> int{
        unsafe { lua::lua_pushinteger(L, 0) };
        1
    }
    pub fn call(&mut self, L: *mut lua_State, _session_id: u32, _flag: u8) -> int{
        unsafe { lua::lua_pushinteger(L, 0) };
        1
    }

}