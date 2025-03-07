#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use std::cell::RefCell;
use std::time::Duration;
use std::rc::{ Rc, Weak };
use std::collections::HashMap;

use mio::net::TcpStream;
use mio::{ Events, Interest, Poll, Token };

use crate::socket_stream::SocketStream;
use crate::socket_listener::SocketListener;

pub type AcceptFunction     = fn(token: u32);
pub type ErrorFunction      = fn(error: &str);
pub type PackageFunction    = fn(data: &[u8]);
pub type ConnectFunction    = fn(ok: bool, reason: &str);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LinkStatus {
    LinkInit        = 0,
    LinkConnecting  = 1,
    LinkConnected   = 2,
    LinkClosing     = 3,
    LinkClosed      = 4,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Prototype {
    ProtoPb         = 0,    // pb协议，pb
    ProtoRpc        = 1,    // rpc协议，rpc
    ProtoText       = 2,    // text协议，mysql/mongo/http/wss/redis
    ProtoMax        = 3,    // max
}

impl From<isize> for Prototype {
    fn from(val: isize) -> Prototype {
        match val {
            0 => Prototype::ProtoPb,
            1 => Prototype::ProtoRpc,
            2 => Prototype::ProtoText,
            _ => Prototype::ProtoMax,
        }
    }
}

impl From<Prototype> for isize {
    fn from(val: Prototype) -> isize {
        match val {
            Prototype::ProtoPb => 0,
            Prototype::ProtoRpc => 1,
            Prototype::ProtoText => 2,
            Prototype::ProtoMax => 3,
        }
    }
}

pub trait SocketObj {
    fn close(&mut self);
    fn do_recv(&mut self);
    fn do_send(&mut self) {}
    fn get_token(&self) -> u32;
    fn update(&mut self, now: u64) -> bool;
    fn is_same_kind(&self, kind: u32)-> bool;
    fn send(&mut self, data: &[u8]) {}
    fn sendv(&mut self, items: &Vec<&[u8]>) {}
    fn set_nodelay(&mut self, flag: bool) {}
    fn set_timeout(&mut self, duration: u64) {}
    fn set_error_callback(&mut self, callback: ErrorFunction) {}
    fn set_accept_callback(&mut self, callback: AcceptFunction) {}
    fn set_connect_callback(&mut self, callback: ConnectFunction) {}
    fn set_package_callback(&mut self, callback: PackageFunction) {}
}

pub struct SocketMgr {
    m_poll: Poll,
    m_events: Events,
    m_max_count: usize,
    self_ref: Weak<RefCell<SocketMgr>>,
    m_objects: HashMap<u32, Box<dyn SocketObj>>,
}

impl SocketMgr {
    pub fn new(max_conn: usize) -> Rc<RefCell<SocketMgr>> {
        let rc = Rc::new(RefCell::new(SocketMgr {
            self_ref: Weak::new(),
            m_max_count: max_conn,
            m_objects: HashMap::new(),
            m_poll: Poll::new().unwrap(),
            m_events: Events::with_capacity(max_conn),
        }));
        rc.borrow_mut().self_ref = Rc::downgrade(&rc);
        rc
    }

    pub fn wait(&mut self, _now: u64, timeout: u64) -> u32 {
        let now = luakit::now();
        self.m_objects.retain(|_, obj| {
            !obj.update(now)
        });
        let mut count = 0;
        let escape = luakit::steady_ms() - now;
        let timeout = if escape >= timeout {0} else {timeout - escape};
        if let Ok(_) = self.m_poll.poll(&mut self.m_events, Some(Duration::from_millis(timeout))) {
            for event in self.m_events.iter() {
                count += 1;
                let token = usize::from(event.token()) as u32;
                if let Some(obj) = self.m_objects.get_mut(&token) {
                    if event.is_readable() { obj.do_recv(); }
                    if event.is_writable() { obj.do_send(); }
                }
            }
        }
        count
    }
    
    pub fn listen(&mut self, ip: String, port: u32) -> Result<u32, String> {
        let mut listener = SocketListener::new(self.self_ref.clone());
        match listener.listen(ip, port) {
            Ok(fd) => {
                if let Some(ref mut sock) = listener.socket {
                    match self.m_poll.registry().register(sock, Token(fd as usize), Interest::READABLE) {
                        Ok(_) => { 
                            self.m_objects.insert(fd, Box::new(listener));
                            return Ok(fd);
                        },
                        Err(e) => {
                            return Err(e.to_string());
                        }
                    }
                }
                Err("listen socket error".to_string())
            },
            Err(e) => {
                Err(e.to_string())
            }
        }
    }

    pub fn connect(&mut self, ip: String, port: u32, timeout: u64) -> Result<u32, String> {
        if self.is_full() {
            return Err("socket mgr is full".to_string());
        }
        let mut connector = SocketStream::new(self.self_ref.clone());
        match connector.connect(ip, port, timeout) {
            Ok(fd) => {
                if let Some(ref mut sock) = connector.socket {
                    match self.m_poll.registry().register(sock, Token(fd as usize), Interest::WRITABLE) {
                        Ok(_) => {
                            self.m_objects.insert(fd, Box::new(connector));
                            return Ok(fd);
                        },
                        Err(e) => {
                            return Err(e.to_string());
                        }
                    }
                }
                Err("connect socket error".to_string())
            },
            Err(e) => {
                Err(e.to_string())
            }
        }
    }

    pub fn watch_connected(&mut self, token: u32, socket: &mut TcpStream) -> Result<u32, String> {
        match self.m_poll.registry().register(socket, Token(token as usize), Interest::READABLE) {
            Ok(_) => return Ok(token),
            Err(e) => return Err(e.to_string()),
        }
    }

    pub fn watch_send(&mut self, token: u32, socket: &mut TcpStream) -> Result<u32, String> {
        match self.m_poll.registry().register(socket, Token(token as usize), Interest::READABLE | Interest::WRITABLE) {
            Ok(_) => return Ok(token),
            Err(e) => return Err(e.to_string()),
        }
    }

    pub fn watch_accepted(&mut self, token: u32, mut socket: TcpStream) -> Result<u32, String> {
        match self.m_poll.registry().register(&mut socket, Token(token as usize), Interest::READABLE | Interest::WRITABLE) {
            Err(e) => return Err(e.to_string()),
            Ok(_) => {
                let stream = SocketStream::load(socket, token, self.self_ref.clone());
                self.m_objects.insert(token, Box::new(stream));
                Ok(token)
            }
        }
    }

    pub fn send(&mut self, token: u32, data: &[u8]) {
        if let Some(obj) = self.m_objects.get_mut(&token) {
            obj.send(data);
        }
    }

    pub fn sendv(&mut self, token: u32, items: &Vec<&[u8]>) {
        if let Some(obj) = self.m_objects.get_mut(&token) {
            obj.sendv(items);
        }
    }

    
    pub fn broadcast(&mut self, kind: u32, data: &[u8]) {
        for (_, obj) in self.m_objects.iter_mut() {
            if obj.is_same_kind(kind) {
                obj.send(data);
            }
        }
    }

    pub fn broadcast_group(&mut self, groups: Vec<u32>, data: &[u8]) {
        for token in groups {
            self.send(token, data);
        }
    }

    pub fn get_object(&self, token: u32) -> Option<&Box<dyn SocketObj>> {
        self.m_objects.get(&token)
    }
    
    pub fn close(&mut self, token: u32) {
        if let Some(obj) = self.m_objects.get_mut(&token) {
            obj.close();
        }
    }

    pub fn set_timeout(&mut self, token: u32, duration: u64) {
        if let Some(obj) = self.m_objects.get_mut(&token) {
            obj.set_timeout(duration);
        }
    }

    pub fn set_nodelay(&mut self, token: u32, flag: bool) {
        if let Some(obj) = self.m_objects.get_mut(&token) {
            obj.set_nodelay(flag);
        }
    }

    fn set_error_callback(&mut self, token: u32, callback: ErrorFunction) {
        if let Some(obj) = self.m_objects.get_mut(&token) {
            obj.set_error_callback(callback);
        }
     }
    fn set_accept_callback(&mut self, token: u32, callback: AcceptFunction) {
        if let Some(obj) = self.m_objects.get_mut(&token) {
            obj.set_accept_callback(callback);
        }
    }
    fn set_connect_callback(&mut self, token: u32, callback: ConnectFunction) {
        if let Some(obj) = self.m_objects.get_mut(&token) {
            obj.set_connect_callback(callback);
        }
    }
    fn set_package_callback(&mut self, token: u32, callback: PackageFunction) {
        if let Some(obj) = self.m_objects.get_mut(&token) {
            obj.set_package_callback(callback);
        }
    }

    pub fn is_full(&self) -> bool {
        return self.m_objects.len() >= self.m_max_count;
    }
}