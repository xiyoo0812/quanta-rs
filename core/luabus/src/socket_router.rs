#![allow(non_snake_case)]
#![allow(dead_code)]

use std::mem;
use std::cell::RefCell;
use std::rc::{ Rc, Weak };

use crate::socket_mgr::SocketMgr;

pub enum RpcType {
    RemoteCall,
    TransferCall,
    ForwardTarget,
    ForwardMaster,
    ForwardBroadcast,
    ForwardHash,
}

#[derive(Clone, Copy)]
struct ServiceNode {
    id: u32,
    token: u32,
    group: u16,
    region: u16,
}

struct ServiceList {
    master: ServiceNode,
    nodes: Vec<ServiceNode>,
}

#[repr(packed)]
pub struct RouterHeaader {
    len: u32,
    context: u8,        //高4位为msg_id，低4位为flag
    session_id: u32,
    target_id: u32,
}

impl RouterHeaader {
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self as *const Self as *const u8,
                mem::size_of_val(self)
            )
        }
    }
}

#[repr(packed)]
pub struct TransferHeader {
    len: u32,
    context: u8,        //高4位为msg_id，低4位为flag
    session_id: u32,
    target_id: u32,
    service_id: u8,
}

impl TransferHeader {
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self as *const Self as *const u8,
                mem::size_of_val(self)
            )
        }
    }
}

pub struct SocketRouter {
    router_count: u32,
    services: Vec<ServiceList>,
    sockmgr: Weak<RefCell<SocketMgr>>,
}

fn get_service_id(node_id: u32) -> usize { 
    (node_id as usize >> 16) & 0xff
}

impl SocketRouter {
    pub fn new(mgr: Weak<RefCell<SocketMgr>>) -> Rc<RefCell<SocketRouter>> {
        Rc::new(RefCell::new(SocketRouter {
            sockmgr: mgr,
            router_count: 0,
            services: (0 ..= u8::MAX)
                .map(|_| ServiceList {
                    master: ServiceNode { id: 0, token: 0, group: 0, region: 0 },
                    nodes: Vec::new(),
                }).collect(),
        }))
    }

    pub fn map_token(&mut self, node_id: u32, token: u32) -> u32 {
        let service_id = get_service_id(node_id);
        let list = &mut self.services[service_id];
        let nodes = &mut list.nodes;
        let idx = nodes.partition_point(|node| node.id < node_id);
        if idx < nodes.len() && nodes[idx].id == node_id {
            if token > 0 {
                nodes[idx].token = token;
            } else {
                nodes.remove(idx);
            }
            return self.choose_master(service_id);
        }
        let node = ServiceNode { id: node_id, token: token, group: 0, region: 0 };
        nodes.insert(idx, node);
        return self.choose_master(service_id);
    }

    pub fn erase(&mut self, node_id: u32) {
        let service_id = get_service_id(node_id);
        let list = &mut self.services[service_id];
        let nodes = &mut list.nodes;
        let idx = nodes.partition_point(|node| node.id < node_id);
        if idx < nodes.len() && nodes[idx].id == node_id {
            nodes.remove(idx);
            self.choose_master(service_id);
        }
    }

    pub fn choose_master(&mut self, service_id: usize) -> u32 {
        let list = &mut self.services[service_id];
        let nodes = &mut list.nodes;
        if nodes.is_empty() {
            list.master = ServiceNode { id: 0, token: 0, group: 0, region: 0 };
            return 0;
        }
        list.master = nodes[0].clone();
        return list.master.id;
    }

    pub fn do_forward_target(&mut self, header: &mut RouterHeaader, data: &[u8]) ->bool {
        let target_id = header.target_id;
        let service_id = get_service_id(target_id);
        let list = &mut self.services[service_id];
        let nodes = &mut list.nodes;
        let idx = nodes.partition_point(|node| node.id < target_id);
        if idx >= nodes.len() && nodes[idx].id != target_id {
            return false;
        }
        if let Some(sockmgr) = self.sockmgr.upgrade() {
            let flag = header.context & 0xf;
            header.context = (RpcType::RemoteCall as u8) << 4 | flag;
            sockmgr.borrow_mut().sendv(nodes[idx].token, &vec![header.as_bytes(), data]);
            self.router_count += 1;
            return true;
        }
        false
    }

    pub fn do_forward_master(&mut self, header: &mut RouterHeaader, data: &[u8]) ->bool {
        let service_id = header.target_id as usize;
        let token = self.services[service_id].master.token;
        if token == 0 {
            return false;
        }
        if let Some(sockmgr) = self.sockmgr.upgrade() {
            let flag = header.context & 0xf;
            header.context = (RpcType::RemoteCall as u8) << 4 | flag;
            sockmgr.borrow_mut().sendv(token, &vec![header.as_bytes(), data]);
            self.router_count += 1;
            return true;
        }
        false
    }
    pub fn do_forward_hash(&mut self, header: &mut RouterHeaader, data: &[u8]) ->bool {
        let hash = (header.target_id & 0xffff) as usize;
        let service_id = get_service_id(header.target_id);
        let list = &mut self.services[service_id];
        let nodes = &mut list.nodes;
        let count = nodes.len();
        if count == 0 {
            return false;
        }
        let token = nodes[hash % count].token;
        if token == 0 {
            return false;
        }
        if let Some(sockmgr) = self.sockmgr.upgrade() {
            let flag = header.context & 0xf;
            header.context = (RpcType::RemoteCall as u8) << 4 | flag;
            sockmgr.borrow_mut().sendv(token, &vec![header.as_bytes(), data]);
            self.router_count += 1;
            return true;
        }
        false
    }

    pub fn do_forward_broadcast(&mut self, header: &mut RouterHeaader, source: u32, data: &[u8], broadcast_num: &mut u32) ->bool {
        let service_id = header.target_id as usize;
        let list = &mut self.services[service_id];
        if list.nodes.is_empty() {
            return false;
        }
        if let Some(sockmgr) = self.sockmgr.upgrade() {
            let flag = header.context & 0xf;
            header.context = (RpcType::RemoteCall as u8) << 4 | flag;
            let send_data = vec![header.as_bytes(), data];
            let nodes = &mut list.nodes;
            for node in nodes.iter() {
                if node.token != 0 && node.token != source {
                    sockmgr.borrow_mut().sendv(node.token, &send_data);
                    self.router_count += 1;
                    *broadcast_num += 1;
                }
            }
        }
        return *broadcast_num > 0;
    }

    pub fn get_route_count(&mut self) -> u32 { 
        let old = self.router_count;
        self.router_count = 0;
        old
    }
}
