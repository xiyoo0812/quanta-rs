#![allow(non_snake_case)]
#![allow(dead_code)]

use lua::lua_State;
use libc::c_int as int;

use luakit::{ PtrBox, LuaPush, LuaGc };

use std::thread;
use std::io::{self, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream, ToSocketAddrs};

use crate::socket_helper::*;

pub struct SocketTcp {
    pub recv_buf: Vec<u8>,
    pub stream: Option<TcpStream>,
    pub listener: Option<TcpListener>,
}

impl LuaGc for SocketTcp {}

impl SocketTcp {
    pub fn new() -> SocketTcp {
        SocketTcp {
            stream: None,
            listener: None,
            recv_buf: vec![0; SOCKET_RECV_LEN],
        }
    }

    pub fn load(stream: TcpStream) -> SocketTcp {
        SocketTcp {
            listener: None,
            stream: Some(stream),
            recv_buf: vec![0; SOCKET_RECV_LEN],
        }
    }

    pub fn close(&mut self) {
        if let Some(socket) =  &self.stream {
            let _ = socket.shutdown(Shutdown::Both);
            return;
        }
    }
    
    pub fn invalid(&self) -> bool {
        self.listener.is_none() && self.stream.is_none()
    }

    pub fn set_no_block(&mut self) {
        if let Some(socket) =  &self.stream {
            let _ = socket.set_nonblocking(true);
            return;
        }
        if let Some(socket) =  &self.listener {
            let _ = socket.set_nonblocking(true);
        }
    }

    pub fn accept(&mut self, L: *mut lua_State, timeout: u64) -> int {
        let listener = match &self.listener {
            Some(listener) => listener,
            None => {
                unsafe { lua::lua_pushnil(L) };
                lua::lua_pushstring(L, "socket invalid");
                return 2;
            }
        };
        match listener.accept() {
            Ok((socket, _)) => {
                let strrem = PtrBox::new(SocketTcp::load(socket));
                strrem.native_to_lua(L);
                1
            },
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(std::time::Duration::from_millis(timeout));
                unsafe { lua::lua_pushboolean(L, 0) };
                lua::lua_pushstring(L, "timeout");
                return 2;
            }
            Err(e) => {
                unsafe { lua::lua_pushnil(L) };
                lua::lua_pushstring(L, &e.to_string());
                return 2;
            }
        }
    }

    pub fn listen(&mut self, L: *mut lua_State, ip: String, port: u32) -> int {
        match TcpListener::bind(format!("{}:{}", ip, port)) {
            Ok(listener) => {
                self.listener = Some(listener);
            },
            Err(e) => {
                unsafe { lua::lua_pushboolean(L, 0) };
                lua::lua_pushstring(L, &e.to_string());
                return 2;
            }
        }
        unsafe { lua::lua_pushboolean(L, 1) };
        1
    }
    
    pub fn connect(&mut self, L: *mut lua_State, ip: String, port: u32, timeout: u64) -> int {
        let addr_str = format!("{}:{}", ip, port);
        let addr = match addr_str.to_socket_addrs() {
            Ok(mut addrs) => addrs.next().unwrap(),
            Err(e) => {
                unsafe { lua::lua_pushboolean(L, 0) };
                lua::lua_pushstring(L, &e.to_string());
                return 2;
            }
        };
        match TcpStream:: connect_timeout(&addr, std::time::Duration::from_millis(timeout)) {
            Ok(stream) => {
                let _ = stream.set_nonblocking(true);
                self.stream = Some(stream);
            },
            Err(e) => {
                unsafe { lua::lua_pushboolean(L, 0) };
                lua::lua_pushstring(L, &e.to_string());
                return 2;
            }
        }
        unsafe { lua::lua_pushboolean(L, 1) };
        1
    }

    pub fn send(&mut self, L: *mut lua_State, buff: Vec<u8>, timeout: u64) -> int {
        let mut socket = match &self.stream {
            Some(socket) => socket,
            None => {
                unsafe { lua::lua_pushboolean(L, 0) };
                lua::lua_pushstring(L, "socket invalid");
                return 2;
            }
        };
        let _ = socket.set_write_timeout(Some(std::time::Duration::from_millis(timeout)));
        match socket.write(&buff) {
            Ok(n) => {
                unsafe { 
                    lua::lua_pushboolean(L, 1);
                    lua::lua_pushinteger(L, n as isize);
                };
                return 2;
            },
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                unsafe { lua::lua_pushboolean(L, 0) };
                lua::lua_pushstring(L, "timeout");
                return 2;
            }
            Err(e) => {
                unsafe { lua::lua_pushboolean(L, 0) };
                lua::lua_pushstring(L, &e.to_string());
                return 2;
            }
        }
    }

    pub fn recv(&mut self, L: *mut lua_State, timeout: u64) -> int {
        let mut socket = match &self.stream {
            Some(socket) => socket,
            None => {
                unsafe { lua::lua_pushboolean(L, 0) };
                lua::lua_pushstring(L, "socket invalid");
                return 2;
            }
        };
        let _ = socket.set_read_timeout(Some(std::time::Duration::from_millis(timeout)));
        match socket.read(&mut self.recv_buf) {
            Ok(n) => {
                unsafe { 
                    lua::lua_pushboolean(L, 1);
                    lua::lua_pushlstring_(L, self.recv_buf.as_ptr() as *const i8, n);
                };
                return 2;
            },
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                unsafe { lua::lua_pushboolean(L, 0) };
                lua::lua_pushstring(L, "timeout");
                return 2;
            }
            Err(e) => {
                self.close();
                unsafe { lua::lua_pushboolean(L, 0) };
                lua::lua_pushstring(L, &e.to_string());
                return 2;
            }
        }
    }

}
