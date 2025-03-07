#![allow(non_snake_case)]
#![allow(dead_code)]

use std::rc::Weak;
use std::cell::RefCell;
use std::net::SocketAddr;

use mio::net::TcpStream;

use crate::socket_helper::get_fd;
use crate::socket_mgr::{ SocketObj, SocketMgr, LinkStatus, ConnectFunction, ErrorFunction, PackageFunction };

pub struct SocketStream {
    pub token: u32,
    pub timeout: u64,
    pub connect_time: u64,
    pub lastrecv_time: u64,
    pub status: LinkStatus,
    pub socket: Option<TcpStream>,
    pub error_cb: ErrorFunction,
    pub connect_cb: ConnectFunction,
    pub package_cb: PackageFunction,
    pub sockmgr: Weak<RefCell<SocketMgr>>,
}

impl SocketStream {
    pub fn new(mgr: Weak<RefCell<SocketMgr>>) -> SocketStream {
        SocketStream {
            token: 0,
            socket: None,
            sockmgr: mgr,
            timeout: 0,
            connect_time: 0,
            lastrecv_time: 0,
            status: LinkStatus::LinkInit,
            connect_cb: |_,_|{},
            package_cb: |_|{},
            error_cb: |_|{}
        }
    }

    pub fn load(sock: TcpStream, token: u32, mgr: Weak<RefCell<SocketMgr>>) -> SocketStream {
        SocketStream {
            token: token,
            sockmgr: mgr,
            timeout: 0,
            connect_time: 0,
            lastrecv_time: 0,
            socket: Some(sock),
            status: LinkStatus::LinkInit,
            connect_cb: |_,_|{},
            package_cb: |_|{},
            error_cb: |_|{}
        }
    }

    pub fn connect(&mut self, ip: String, port: u32, timeout: u64) -> Result<u32, String> {
        let addr_str = format!("{}:{}", ip, port);
        let addr: SocketAddr = match addr_str.parse() {
            Ok(addr) => addr,
            Err(e) => return Err(e.to_string()),
        };
        match TcpStream::connect(addr) {
            Ok(stream) => {
                self.status = LinkStatus::LinkConnecting;
                self.token = get_fd(&stream);
                self.connect_time = timeout;
                self.socket = Some(stream);
                Ok(self.token)
            },
            Err(e) => {
                Err(e.to_string())
            }
        }
    }

    
    fn send_impl(&mut self) {

    }

    fn recv_impl(&mut self) {
    }

    fn on_error(&mut self, err: &str) {
        if self.status == LinkStatus::LinkConnected {
            self.status = LinkStatus::LinkClosed;
            (self.error_cb)(err);
        }
    }

    fn on_connect(&mut self, ok: bool,  err: &str) {
        if ok {
            self.lastrecv_time = luakit::steady_ms();
            self.status = LinkStatus::LinkConnected;
        } else {
            self.status = LinkStatus::LinkClosed;
        }
        (self.connect_cb)(ok, err);
    }
}

impl SocketObj for SocketStream {
    fn close(&mut self) { }
    fn send(&mut self, _data: &[u8]) {}
    fn sendv(&mut self, _items: &Vec<&[u8]>){}
    fn get_token(&self) -> u32 { self.token }
    fn is_same_kind(&self, kind: u32)-> bool { self.token == kind }
    fn set_timeout(&mut self, duration: u64){ self.timeout = duration; }
    fn set_error_callback(&mut self, callback: ErrorFunction) { self.error_cb = callback; }
    fn set_connect_callback(&mut self, callback: ConnectFunction) { self.connect_cb = callback; }
    fn set_package_callback(&mut self, callback: PackageFunction) { self.package_cb = callback; }
    fn set_nodelay(&mut self, flag: bool){
        if let Some(ref mut stream) = self.socket {
            let _ = stream.set_nodelay(flag);
        }
    }
    fn do_recv(&mut self) {
        if self.status == LinkStatus::LinkConnected || self.status == LinkStatus::LinkClosing {
            self.recv_impl();
        }
    }
    fn do_send(&mut self) {
        if let Some(ref mut stream) = self.socket {
            if self.status == LinkStatus::LinkConnected || self.status == LinkStatus::LinkClosing {
                self.send_impl();
                return;
            }
            if self.status == LinkStatus::LinkConnecting {
                match stream.peer_addr() {
                    Ok(_) => {
                        if let Some(mgr) = self.sockmgr.upgrade() {
                            match mgr.borrow_mut().watch_connected(self.token, stream) {
                                Err(e) => self.on_connect(false, &e.to_string()),
                                Ok(_) => self.on_connect(true, "ok"),
                            }
                        }
                    },
                    Err(e) => self.on_connect(false, &e.to_string()),
                }
            }
        }
    }
    fn update(&mut self, _now: u64) -> bool { true }
}