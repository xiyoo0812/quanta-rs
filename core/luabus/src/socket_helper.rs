#![allow(non_snake_case)]
#![allow(dead_code)]

use std::net::{TcpListener, SocketAddrV4, Ipv4Addr};

pub const SOCKET_RECV_LEN: usize   = 4096;

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