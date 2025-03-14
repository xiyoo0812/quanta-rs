#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use std::rc::Rc;
use std::cell::RefCell;

use lua::lua_State;
use libc::c_int as int;

use crate::socket_router::SocketRouter;
use crate::lua_socket_node::LuaSocketNode;
use crate::socket_mgr::{Prototype, SocketMgr};

use luakit::{ LuaGc, LuaPush, PtrBox };

pub struct LuaSocketMgr {
    lvm: *mut lua_State,
    socket_mgr: Rc<RefCell<SocketMgr>>,
    socket_router: Rc<RefCell<SocketRouter>>,
}

impl LuaGc for LuaSocketMgr {}

impl LuaSocketMgr {
    pub fn new(L: *mut lua_State, max_conn: usize) -> LuaSocketMgr {
        let mgr = SocketMgr::new(max_conn);
        let weak = Rc::downgrade(&mgr);
        LuaSocketMgr {
            socket_router: SocketRouter::new(weak),
            socket_mgr: mgr,
            lvm: L
        }
    }

    pub fn map_token(&self, node_id: u32, token: u32) -> u32 {
        self.socket_router.borrow_mut().map_token(node_id, token)
    }

    pub fn wait(&mut self, now: u64, timeout: u64) -> u32 {
        self.socket_mgr.borrow_mut().wait(now, timeout)
    }
    
    pub fn listen(&mut self, L: *mut lua_State, ip: String, port: u32) -> int {
        match self.socket_mgr.borrow_mut().listen(ip, port) {
            Err(e) => {
                luakit::variadic_return!(L, lua::LUA_NIL, e)
            },
            Ok(token) => {
                let mgr = Rc::downgrade(&self.socket_mgr);
                let router = Rc::downgrade(&self.socket_router);
                let ptype = lua::luaL_optinteger(L, 3, Prototype::ProtoRpc.into());
                let node = LuaSocketNode::new(token, self.lvm, mgr, router, ptype.into());
                luakit::variadic_return!(L, PtrBox::new(node), "ok")
            },
        }
    }

    pub fn connect(&mut self, L: *mut lua_State, ip: String, port: u32, timeout: u64) -> int {
        match self.socket_mgr.borrow_mut().connect(ip, port, timeout) {
            Err(e) => {
                luakit::variadic_return!(L, lua::LUA_NIL, e)
            },
            Ok(token) => {
                let mgr = Rc::downgrade(&self.socket_mgr);
                let router = Rc::downgrade(&self.socket_router);
                let ptype = lua::luaL_optinteger(L, 4, Prototype::ProtoRpc.into());
                let node = LuaSocketNode::new(token, self.lvm, mgr, router, ptype.into());
                luakit::variadic_return!(L, PtrBox::new(node), "ok")
            },
        }
    }
    
    pub fn broadcast(&mut self, kind: u32, data: &[u8]) {
        self.socket_mgr.borrow_mut().broadcast(kind, data);
    }

    pub fn broadgroup(&mut self, groups: Vec<u32>, data: &[u8]) {
        self.socket_mgr.borrow_mut().broadcast_group(groups, data);
    }
}