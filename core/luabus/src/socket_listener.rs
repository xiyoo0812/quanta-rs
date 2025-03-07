#![allow(non_snake_case)]
#![allow(dead_code)]

use std::rc::Weak;
use std::cell::RefCell;
use std::net::SocketAddr;

use mio::net::TcpListener;

use crate::socket_helper::get_fd;
use crate::socket_mgr::{ SocketObj, SocketMgr, LinkStatus, ErrorFunction, AcceptFunction };

pub struct SocketListener {
    pub token: u32,
    pub status: LinkStatus,
    pub error_cb: ErrorFunction,
    pub accept_cb: AcceptFunction,
    pub socket: Option<TcpListener>,
    pub sockmgr: Weak<RefCell<SocketMgr>>,
}

impl SocketListener {
    pub fn new(mgr: Weak<RefCell<SocketMgr>>) -> SocketListener {
        SocketListener {
            token: 0,
            socket: None,
            sockmgr: mgr,
            error_cb: |_|{},
            accept_cb: |_|{},
            status: LinkStatus::LinkInit,
        }
    }

    pub fn listen(&mut self, ip: String, port: u32) -> Result<u32, String> {
        let addr_str = format!("{}:{}", ip, port);
        let addr: SocketAddr = match addr_str.parse() {
            Ok(addr) => addr,
            Err(e) => return Err(e.to_string()),
        };
        match TcpListener::bind(addr) {
            Ok(listener) => {
                self.status = LinkStatus::LinkConnected;
                self.token = get_fd(&listener);
                self.socket = Some(listener);
                Ok(self.token)
            },
            Err(e) => {
                Err(e.to_string())
            }
        }
    }

    fn on_error(&mut self, err: &str) {
        if self.status == LinkStatus::LinkConnected {
            self.status = LinkStatus::LinkClosed;
            (self.error_cb)(err);
        }
    }

}

impl SocketObj for SocketListener {
    fn close(&mut self) { }
    fn get_token(&self) -> u32 { self.token }
    fn is_same_kind(&self, kind: u32)-> bool { return self.token == kind; }
    fn set_error_callback(&mut self, callback: ErrorFunction) { self.error_cb = callback; }
    fn set_accept_callback(&mut self, callback: AcceptFunction) { self.accept_cb = callback; }
    fn do_recv(&mut self) {
        
        if let Some(ref mut listener) = self.socket {
            if self.status == LinkStatus::LinkConnected {
                match listener.accept() {
                    Ok((socket, _addr)) => {
                        if let Some(mgr) = self.sockmgr.upgrade() {
                            match mgr.borrow_mut().watch_accepted(get_fd(&socket), socket) {
                                Err(e) => self.on_error(&e.to_string()),
                                Ok(token) => (self.accept_cb)(token),
                            }
                        }
                    }
                    Err(e) => self.on_error(&e.to_string()),
                }

            }
        }
    }
    
    fn update(&mut self, _now: u64) -> bool { true }
}