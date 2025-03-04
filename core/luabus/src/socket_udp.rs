#![allow(non_snake_case)]
#![allow(dead_code)]

use lua::lua_State;
use libc::c_int as int;

use std::io;
use std::net::UdpSocket;

use luakit::LuaGc;

use crate::socket_helper::*;

pub struct SocketUdp {
    pub recv_buf: Vec<u8>,
    pub socket: Option<UdpSocket>,
}

impl LuaGc for SocketUdp {}

impl SocketUdp {
    pub fn new() -> SocketUdp {
        SocketUdp { 
            socket: None,
            recv_buf: vec![0; SOCKET_RECV_LEN],
        }
    }

    pub fn close(&self) {
    }
    
    pub fn set_no_block(&mut self) {
        if let Some(socket) =  &self.socket {
            let _ = socket.set_nonblocking(true);
        }
    }

    pub fn bind(&mut self, L: *mut lua_State, ip: String, port: u32, noblock:bool) -> int {
        match UdpSocket::bind(format!("{}:{}", ip, port)) {
            Ok(socket) => {
                self.socket = Some(socket);
            },
            Err(e) => {
                unsafe { lua::lua_pushboolean(L, 0) };
                lua::lua_pushstring(L, &e.to_string());
                return 2;
            }
        }
        if noblock {
            self.set_no_block();
        }
        unsafe { lua::lua_pushboolean(L, 1) };
        1
    }

    pub fn send(&mut self, L: *mut lua_State, buff: Vec<u8>, _size: usize, ip: String, port: u32) -> int {
        let socket = match &self.socket {
            Some(socket) => socket,
            None => {
                unsafe { lua::lua_pushboolean(L, 0) };
                lua::lua_pushstring(L, "socket invalid");
                return 2;
            }
        };
        match socket.send_to(&buff, format!("{}:{}", ip, port)) {
            Ok(n) => {
                unsafe {
                    lua::lua_pushboolean(L, 1);
                    lua::lua_pushinteger(L, n as isize);
                };
                return 2;
            },
            Err(e) => {
                unsafe {
                    lua::lua_pushboolean(L, 0);
                    lua::lua_pushstring(L, &e.to_string());
                    lua::lua_pushinteger(L, e.kind() as isize);
                };
                return 3;
            }
        }
    }

    pub fn recv(&mut self, L: *mut lua_State) -> int {
        let socket = match &self.socket {
            Some(socket) => socket,
            None => {
                unsafe { lua::lua_pushboolean(L, 0) };
                lua::lua_pushstring(L, "socket invalid");
                return 2;
            }
        };
        match socket.recv_from(&mut self.recv_buf) {
            Ok((n, addr)) => {
                unsafe { 
                    lua::lua_pushboolean(L, 1);
                    lua::lua_pushlstring_(L, self.recv_buf.as_ptr() as *const i8, n);
                    lua::lua_pushlstring(L, &addr.ip().to_string());
                    lua::lua_pushinteger(L, addr.port() as isize);
                };
                return 4;
            },
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                unsafe { lua::lua_pushboolean(L, 0) };
                lua::lua_pushstring(L, "EWOULDBLOCK");
                unsafe { lua::lua_pushinteger(L, e.kind() as isize) };
                return 3;
            }
            Err(e) => {
                self.close();
                unsafe { lua::lua_pushboolean(L, 0) };
                lua::lua_pushstring(L, &e.to_string());
                unsafe { lua::lua_pushinteger(L, e.kind() as isize) };
                return 3;
            }
        }
    }
}