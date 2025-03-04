#![allow(non_snake_case)]

extern crate lua;
extern crate libc;
extern crate luakit;

mod socket_dns;
mod socket_tcp;
mod socket_udp;
mod socket_ping;
mod socket_helper;

use lua::lua_State;
use libc::c_int as int;

use luakit::{ Luakit, PtrBox, LuaPush, LuaPushFn, LuaPushFnMut, LuaPushLuaFn, LuaPushLuaFnMut, };

use socket_tcp::SocketTcp;
use socket_udp::SocketUdp;

#[no_mangle]
pub extern "C" fn luaopen_luabus(L: *mut lua_State) -> int {
    let mut kit = Luakit::load(L);
    let mut luabus = kit.new_table(Some("luabus"));
    luakit::set_function!(luabus, "host", socket_dns::gethostip);
    luakit::set_function!(luabus, "dns", socket_dns::gethostbydomain);
    luakit::set_function!(luabus, "ping", socket_ping::socket_ping);
    luakit::set_function!(luabus, "derive_port", socket_helper::derive_port);
    luakit::set_function!(luabus, "udp", || { PtrBox::new(SocketUdp::new()) });
    luakit::set_function!(luabus, "tcp", || { PtrBox::new(SocketTcp::new()) });
    luakit::new_class!(SocketUdp, luabus, "SocketUdp",
        "send", SocketUdp::send,
        "recv", SocketUdp::recv,
        "bind", SocketUdp::bind,
        "close", SocketUdp::close,
        "set_no_block", SocketUdp::set_no_block
    );
    luakit::new_class!(SocketTcp, luabus, "SocketTcp",
        "send", SocketTcp::send,
        "recv", SocketTcp::recv,
        "close", SocketTcp::close,
        "listen", SocketTcp::listen,
        "accept", SocketTcp::accept,
        "connect", SocketTcp::connect,
        "invalid", SocketTcp::invalid,
        "set_no_block", SocketTcp::set_no_block
    );
    luabus.native_to_lua(L)
}
