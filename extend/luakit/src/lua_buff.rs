#![allow(non_snake_case)]
#![allow(dead_code)]

use lua::ternary;

use crate::lua_slice::Slice;
use std::{mem, ops::{Deref, DerefMut}};

const BUFFER_DEF: usize = 64 * 1024;       // 64K
const BUFFER_MAX: usize = 16 * 1024 * 1024;// 16M
const ALIGN_SIZE: usize = 16;              // 水位

#[derive(Debug)]
pub struct LuaBuf {
    head: usize,
    tail: usize,
    data: Vec<u8>,
    capacity: usize,
    max_capacity: usize,
}

impl LuaBuf {
    pub fn new() -> Self {
        Self {
            head: 0,
            tail: 0,
            capacity: BUFFER_DEF,
            data: vec![0; BUFFER_DEF],
            max_capacity: BUFFER_DEF * ALIGN_SIZE
        }
    }

    pub fn reset(&mut self) {
        if self.capacity != BUFFER_DEF {
            self.data.resize(BUFFER_DEF, 0);
            self.capacity = BUFFER_DEF;
        }
        self.head = 0;
        self.tail = 0;
    }

    pub fn size(&self) -> usize {
        self.tail - self.head
    }

    pub fn empty(&self) -> bool {
        self.head == self.tail
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn clean(&mut self) {
        let data_len = self.size();
        if self.capacity > self.max_capacity && data_len < BUFFER_DEF {
            self.resize(self.capacity / 2);
        }
        self.head = 0;
        self.tail = 0;
    }

    pub fn copy(&mut self, offset: usize, src: &[u8]) -> usize {
        if offset + src.len() <= self.size() {
            let start = self.head + offset;
            self.data[start..start+src.len()].copy_from_slice(src);
            src.len()
        } else {
            0
        }
    }

    pub fn push_data(&mut self, src: &[u8]) -> usize {
        if let Some(_) = self.peek_space(src.len()) {
            let start = self.tail;
            self.data[start..start+src.len()].copy_from_slice(src);
            self.tail += src.len();
            src.len()
        } else {
            0
        }
    }

    pub fn pop_data(&mut self, dest: &mut [u8]) -> usize {
        let pop_len = dest.len().min(self.size());
        if pop_len > 0 {
            let start = self.head;
            dest[..pop_len].copy_from_slice(&self.data[start..start+pop_len]);
            self.head += pop_len;
            pop_len
        } else {
            0
        }
    }

    pub fn pop_size(&mut self, erase_len: usize) -> usize {
        let erase_len = erase_len.min(self.size());
        if erase_len > 0 {
            self.head += erase_len;
            if self.capacity > self.max_capacity && self.size() < BUFFER_DEF {
                self.regularize();
                self.resize(self.capacity / 2);
            }
            erase_len
        } else {
            0
        }
    }

    pub fn peek_data<'a>(&'a self, peek_len: usize, offset: Option<usize>) -> Option<&'a [u8]> {
        let offset = offset.unwrap_or(0);
        let available = self.size().saturating_sub(offset);
        (peek_len <= available).then(|| {
            let start = self.head + offset;
            &self.data[start..start+peek_len]
        })
    }

    pub fn get_slice<'a>(&'a self, len: Option<usize>, offset: Option<usize>) -> Slice<'a> {
        let len = len.unwrap_or(0);
        let offset = offset.unwrap_or(0);
        let data_len = self.tail - (self.head + offset);
        let len = ternary!(len == 0, data_len, ternary!(len > data_len, data_len, len));
        Slice::attach(&self.data[self.head + offset..self.head + offset + len])
    }

    pub fn peek_space(&mut self, need_len: usize) -> Option<&mut [u8]> {
        if self.free_space() < need_len {
            self.regularize();
            if self.free_space() < need_len {
                let mut new_cap = self.capacity * 2;
                while new_cap - self.size() < need_len {
                    new_cap *= 2;
                    if new_cap > BUFFER_MAX {
                        return None;
                    }
                }
                self.resize(new_cap);
            }
        }
        let start = self.tail;
        Some(&mut self.data[start..start+need_len])
    }

    pub fn data<'a>(&'a self) -> &'a [u8] {
        &self.data
    }

    pub fn string(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.data[self.head..self.tail])}
    }

    pub fn write_char(&mut self, value: &[u8]) -> usize {
        return self.push_data(value);
    }

    pub fn write_str(&mut self, value: &str) -> usize {
        return self.push_data(value.as_bytes());
    }

    pub fn write<T: ?Sized>(&mut self, value: &T) -> usize {
        let len = mem::size_of_val(value);
        let bytes = unsafe { std::slice::from_raw_parts(value as *const T as *const u8, len) };
        self.push_data(bytes)
    }

    pub fn peek<T>(&self) -> Option<&T> {
        let tpe_len = mem::size_of::<T>();
        let data_len = self.tail - self.head;
        if tpe_len > 0 && data_len >= tpe_len {
            let item_ptr = self.data[self.head..].as_ptr() as *const T;
            unsafe { Some(&*item_ptr) }
        } else {
            None
        }
    }

    pub fn read<T>(&mut self) -> Option<&T> {
        let tpe_len = mem::size_of::<T>();
        let data_len = self.tail - self.head;
        if tpe_len > 0 && data_len >= tpe_len {
            let item_ptr = self.data[self.head..].as_ptr() as *const T;
            self.head += tpe_len;
            unsafe { Some(&*item_ptr) }
        } else {
            None
        }
    }

    // 辅助方法
    pub fn free_space(&self) -> usize {
        self.capacity - self.tail
    }

    fn resize(&mut self, new_cap: usize) {
        if new_cap == self.capacity || new_cap < self.size() || new_cap > BUFFER_MAX {
            return;
        }

        let mut new_data = vec![0; new_cap];
        let data_len = self.size();
        new_data[..data_len].copy_from_slice(&self.data[self.head..self.tail]);
        
        self.data = new_data;
        self.head = 0;
        self.tail = data_len;
        self.capacity = new_cap;
    }

    fn regularize(&mut self) {
        if self.head > 0 {
            let data_len = self.size();
            self.data.copy_within(self.head..self.tail, 0);
            self.head = 0;
            self.tail = data_len;
        }
    }
}

// 为方便使用实现的常见 trait
impl Deref for LuaBuf {
    type Target = [u8];
    
    fn deref(&self) -> &Self::Target {
        &self.data[self.head..self.tail]
    }
}

impl DerefMut for LuaBuf {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let tail = self.tail;
        &mut self.data[tail..]
    }
}
