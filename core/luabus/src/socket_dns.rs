#![allow(non_snake_case)]
#![allow(dead_code)]

use lua::lua_State;
use libc::c_int as int;

use std::net::{IpAddr, UdpSocket, ToSocketAddrs};

use luakit::LuaPush;

pub fn gethostip(L: *mut lua_State) -> int {
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
        if let Ok(_) = socket.connect("8.8.8.8:80") {
            if let Ok(addr) = socket.local_addr() {
                if let IpAddr::V4(addr) = addr.ip() {
                    return addr.to_string().native_to_lua(L);
                }
            }
        };
    }
    lua::lua_pushstring(L, "127.0.0.1");
    1
}

pub fn gethostbydomain(L: *mut lua_State, domain: String) -> int {
    let addr = format!("{}:0", domain);
    if let Ok(addrs) = addr.to_socket_addrs() {
        let addrvec: Vec<String> = addrs.map(|sa| sa.ip().to_string()).collect();
        return luakit::vector_return!(L, addrvec);
    }
    0
}
