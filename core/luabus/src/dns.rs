#![allow(non_snake_case)]
#![allow(dead_code)]

use lua::lua_State;
use libc::c_int as int;

use std::{ mem, ptr };
use std::ffi::CString;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use socket2::{Domain, Protocol, Socket, Type, SockAddr};

use luakit::LuaPush;
unsafe fn resolver_ip(domain: &str, port: u16) -> Option<SocketAddr>{
    None
}

pub fn gethostip(L: *mut lua_State) -> int {
    0
}

