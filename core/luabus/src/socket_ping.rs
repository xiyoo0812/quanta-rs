#![allow(non_snake_case)]
#![allow(dead_code)]

use lua::lua_State;
use libc::c_int as int;

use std::time::Duration;
use ping_rs::PingOptions;
use std::net::{IpAddr, ToSocketAddrs};

fn domain2ip(domain: &str) -> Option<IpAddr> {
    let addr = format!("{}:0", domain);
    if let Ok(addrs) = addr.to_socket_addrs() {
        let addrvec: Vec<IpAddr> = addrs.map(|sa| sa.ip()).collect();
        if addrvec.len() > 0 {
            return Some(addrvec[0])
        }
    }
    None
}

pub fn socket_ping(L: *mut lua_State, domain: String, times: u32) -> int {
    if let Some(addr) = domain2ip(&domain) {
        let mut total = 0u32;
        let data = vec![0u8; 4];
        let timeout = Duration::from_secs(1);
        let options = PingOptions { ttl: 64, dont_fragment: true };
        for _ in 0..times {
            match ping_rs::send_ping(&addr, timeout, &data, Some(&options)) {
                Ok(reply) => total += reply.rtt,
                Err(_) => {
                    unsafe { lua::lua_pushinteger(L, -1)};
                    return 1;
                }
            }
        }
        unsafe { lua::lua_pushnumber(L, total as f64 / times as f64)};
        return 1;
    }
    unsafe { lua::lua_pushinteger(L, -1)};
    1
}