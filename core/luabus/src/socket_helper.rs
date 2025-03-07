#![allow(non_snake_case)]
#![allow(dead_code)]

#[cfg(unix)]
use std::os::unix::io::AsRawFd;
#[cfg(windows)]
use std::os::windows::io::AsRawSocket;

use std::net::{TcpListener, SocketAddrV4, Ipv4Addr};

pub const SOCKET_RECV_LEN: usize   = 4096;

#[cfg(windows)]
pub fn get_fd<T>(sock: &T) -> u32 where T: AsRawSocket {
    return sock.as_raw_socket() as u32;
}
#[cfg(unix)]
pub fn get_fd<T>(sock: &T) -> u32 where T: AsRawFd {
    return sock.as_raw_fd() as u32;
}

pub fn derive_port(mut port: u16) -> u16 {
    for _ in 0..20 {
        let addr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port);
        match TcpListener::bind(addr) {
            Ok(_) => return port,
            Err(_) => port = port.wrapping_add(1)
        }
    }
    0
}