#![allow(non_snake_case)]
#![allow(dead_code)]

use lua::lua_State;
use libc::c_int as int;

use std::cell::RefCell;
use std::collections::LinkedList;

use luakit::LuaPush;

const TIME_NEAR_SHIFT: usize    = 8;
const TIME_LEVEL_SHIFT: usize   = 6;
const TIME_NEAR: usize          = 1 << TIME_NEAR_SHIFT;
const TIME_LEVEL: usize         = 1 << TIME_LEVEL_SHIFT;
const TIME_NEAR_MASK: usize     = TIME_NEAR - 1;
const TIME_LEVEL_MASK: usize    = TIME_LEVEL - 1;

#[derive(Debug)]
struct TimerNode {
    expire: usize,
    timer_id: u64,
}

type IntegerVector = Vec<u64>;
type TimerList = LinkedList<TimerNode>;

struct LuaTimer {
    time: usize,
    nears: [TimerList; TIME_NEAR],
    t: [[TimerList; TIME_LEVEL]; 4],
}

impl LuaTimer {
    pub fn new() -> Self {
        LuaTimer {
            time: 0,
            nears: [(); TIME_NEAR].map(|_| LinkedList::new()),
            t: [
                [(); TIME_LEVEL].map(|_| LinkedList::new()),
                [(); TIME_LEVEL].map(|_| LinkedList::new()),
                [(); TIME_LEVEL].map(|_| LinkedList::new()),
                [(); TIME_LEVEL].map(|_| LinkedList::new()),
            ],
        }
    }

    pub fn update(&mut self, elapse: usize) -> IntegerVector {
        let mut timers = Vec::new();
        self.execute(&mut timers);
        for _ in 0..elapse {
            self.shift();
            self.execute(&mut timers);
        }
        timers
    }

    pub fn insert(&mut self, timer_id: u64, escape: usize) {
        let node = TimerNode {
            expire: self.time + escape,
            timer_id,
        };
        self.add_node(node);
    }

    fn add_node(&mut self, node: TimerNode) {
        let expire = node.expire;
        if (expire | TIME_NEAR_MASK) == (self.time | TIME_NEAR_MASK) {
            self.nears[expire & TIME_NEAR_MASK].push_back(node);
            return;
        }

        let mut mask = TIME_NEAR << TIME_LEVEL_SHIFT;
        for i in 0..3 {
            if (expire | (mask - 1)) == (self.time | (mask - 1)) {
                let idx = (expire >> (TIME_NEAR_SHIFT + i * TIME_LEVEL_SHIFT)) & TIME_LEVEL_MASK;
                self.t[i][idx].push_back(node);
                return;
            }
            mask <<= TIME_LEVEL_SHIFT;
        }
    }

    fn shift(&mut self) {
        self.time += 1;
        if self.time == 0 {
            self.move_list(3, 0);
            return;
        }

        let mut i = 0;
        let ct = self.time;
        let mut mask = TIME_NEAR;
        let mut stime = ct >> TIME_NEAR_SHIFT;
        while (ct & (mask - 1)) == 0 {
            let idx = stime & TIME_LEVEL_MASK;
            if idx != 0 {
                self.move_list(i, idx);
                break;
            }
            mask <<= TIME_LEVEL_SHIFT;
            stime >>= TIME_LEVEL_SHIFT;
            i += 1;
        }
    }

    fn execute(&mut self, timers: &mut IntegerVector) {
        let idx = self.time & TIME_NEAR_MASK;
        while let Some(node) = self.nears[idx].pop_front() {
            timers.push(node.timer_id);
        }
    }

    fn move_list(&mut self, level: usize, idx: usize) {
        let mut nodes_to_move: LinkedList<TimerNode> = LinkedList::new();
        std::mem::swap(&mut self.t[level][idx], &mut nodes_to_move);
        for node in nodes_to_move {
            self.add_node(node);
        }
    }
}

thread_local! {
    static LUA_TIMER: RefCell<LuaTimer> = RefCell::new(LuaTimer::new());
}

pub fn timer_insert(L: *mut lua_State) -> int {
    let timer_id = lua::lua_tointeger(L, 1) as u64;
    let escape = lua::lua_tointeger(L, 2) as usize;
    LUA_TIMER.with(|lua_timer| {
        let mut lua_timer = lua_timer.borrow_mut();
        lua_timer.insert(timer_id, escape);
    });
    return 0;
}

pub fn timer_update(L: *mut lua_State) -> int {
    let escape = lua::lua_tointeger(L, 1) as usize;
    return LUA_TIMER.with(|lua_timer| {
        let mut lua_timer = lua_timer.borrow_mut();
        let timers = lua_timer.update(escape);
        luakit::variadic_return!(L, timers)
    });
}


pub fn timer_sleep(L: *mut lua_State) -> int {
    let ms = lua::lua_tointeger(L, 1) as u64;
    luakit::sleep_ms(ms);
    return 0;
}