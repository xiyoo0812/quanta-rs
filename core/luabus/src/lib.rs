#![allow(non_snake_case)]

extern crate lua;
extern crate libc;
extern crate luakit;

mod socket_dns;
mod socket_tcp;
mod socket_udp;
mod socket_mgr;
mod socket_ping;
mod socket_helper;
mod socket_stream;
mod socket_router;
mod socket_listener;
mod lua_socket_mgr;
mod lua_socket_node;

use lua::lua_State;
use libc::c_int as int;

use luakit::{ Luakit, PtrBox, LuaPush, LuaPushFn, LuaPushFnMut, LuaPushLuaFn, LuaPushLuaFnMut };

use socket_tcp::SocketTcp;
use socket_udp::SocketUdp;
use lua_socket_mgr::LuaSocketMgr;
use lua_socket_node::LuaSocketNode;

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
    luakit::set_function!(luabus, "create_socket_mgr", |L, c| { PtrBox::new(LuaSocketMgr::new(L, c)) });
    luakit::new_enum!(luabus, "eproto_type",
        "pb", socket_mgr::Prototype::ProtoPb,
        "rpc", socket_mgr::Prototype::ProtoRpc,
        "text", socket_mgr::Prototype::ProtoText
    );
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
    luakit::new_class!(LuaSocketMgr, luabus, "LuaSocketMgr",
        "wait", LuaSocketMgr::wait,
        "listen", LuaSocketMgr::listen,
        "connect", LuaSocketMgr::connect,
        "map_token", LuaSocketMgr::map_token,
        "broadcast", LuaSocketMgr::broadcast,
        "broadgroup", LuaSocketMgr::broadgroup
    );
    luakit::new_class!(LuaSocketNode, luabus, "LuaSocketNode",
        "call", LuaSocketNode::call,
        "close", LuaSocketNode::close,
        "call_pb", LuaSocketNode::call_pb,
        "call_data", LuaSocketNode::call_data,
        "set_nodelay", LuaSocketNode::set_nodelay,
        "set_timeout", LuaSocketNode::set_timeout,
        "get_route_count", LuaSocketNode::get_route_count,
        "build_session_id", LuaSocketNode::build_session_id;
        "ip" => ip: String;
        "token" => token: u32;
        "stoken"=> stoken: u32
    );
    luabus.native_to_lua(L)
}
