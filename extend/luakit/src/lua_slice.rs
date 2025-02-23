#![allow(non_snake_case)]
#![allow(dead_code)]

use libc::c_int as int;
use libc::c_char as char;

use lua::{ to_cptr, lua_State };

#[derive(Clone, Debug)]
pub struct Slice<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Slice<'a> {
    pub fn attach(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub fn size(&self) -> usize {
        self.data.len() - self.pos
    }

    pub fn is_empty(&self) -> bool {
        self.pos >= self.data.len()
    }

    pub fn peek(&self, peek_len: usize, offset: usize) -> Option<&[u8]> {
        let start = self.pos + offset;
        let end = start + peek_len;
        (end <= self.data.len()).then(|| &self.data[start..end])
    }

    pub fn erase(&mut self, erase_len: usize) {
        let new_pos = self.pos + erase_len;
        if new_pos <= self.data.len() {
            self.pos = new_pos;
        }
    }

    pub fn touch<T: Copy + Default>(&self) -> Option<T> {
        let size = std::mem::size_of::<T>();
        let end = self.pos + size;
        if end > self.data.len() {
            return None;
        }
        let value = unsafe {
            let ptr = self.data.as_ptr().add(self.pos) as *const T;
            ptr.read_unaligned()
        };
        Some(value)
    }

    pub fn read<T: Copy + Default>(&mut self) -> Option<T> {
        let size = std::mem::size_of::<T>();
        let end = self.pos + size;
        if end > self.data.len() {
            return None;
        }
        let value = unsafe {
            let ptr = self.data.as_ptr().add(self.pos) as *const T;
            ptr.read_unaligned()
        };
        self.pos += size;
        Some(value)
    }

    pub fn data(&self) -> &[u8] {
        &self.data[self.pos..]
    }

    pub fn head(&self) -> *const char {
        to_cptr(&self.data[self.pos..])
    }

    pub fn eof(&mut self) -> &[u8] {
        let data = &self.data[self.pos..];
        self.pos += data.len();
        data
    }

    pub fn contents(&self) -> &'a [u8] {
        &self.data[self.pos..]
    }

    pub fn check(&self, L: *mut lua_State) -> int{
        let peek_len = lua::lua_tointeger(L, 1) as usize;
        if self.size() >= peek_len {
            unsafe { lua::lua_pushlstring_(L, to_cptr(&self.data[self.pos..]), peek_len) };
            return 1;
        }
        0
    }

    pub fn recv(&mut self, L: *mut lua_State) -> int{
        let read_len = lua::lua_tointeger(L, 1) as usize;
        if read_len > 0 && self.size() >= read_len {
            unsafe { lua::lua_pushlstring_(L, to_cptr(&self.data[self.pos..]), read_len) };
            self.pos += read_len;
            return 1;
        }
        0
    }

    pub fn string(&self, L: *mut lua_State) -> int{
        unsafe { lua::lua_pushlstring_(L, to_cptr(&self.data[self.pos..]), self.size()) };
        1
    }

}

