use std::ops::{Deref, DerefMut};

const BUFFER_DEF: usize = 64 * 1024;       // 64K
const BUFFER_MAX: usize = 16 * 1024 * 1024;// 16M
const ALIGN_SIZE: usize = 16;              // 水位

#[derive(Debug)]
pub struct LuaBuf {
    data: Vec<u8>,
    head: usize,
    tail: usize,
    capacity: usize,
    max_capacity: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct Slice<'a> {
    data: &'a [u8],
}

impl LuaBuf {
    pub fn new() -> Self {
        Self {
            data: vec![0; BUFFER_DEF],
            head: 0,
            tail: 0,
            capacity: BUFFER_DEF,
            max_capacity: BUFFER_DEF * ALIGN_SIZE,
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

    pub fn peek_data(&self, peek_len: usize, offset: usize) -> Option<&[u8]> {
        let available = self.size().saturating_sub(offset);
        (peek_len <= available).then(|| {
            let start = self.head + offset;
            &self.data[start..start+peek_len]
        })
    }

    pub fn get_slice(&self, len: usize, offset: usize) -> Option<Slice> {
        self.peek_data(len, offset).map(|data| Slice { data })
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

    pub fn as_string(&self) -> &str {
        std::str::from_utf8(&self.data[self.head..self.tail]).unwrap_or_default()
    }

    // 辅助方法
    fn free_space(&self) -> usize {
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

impl<'a> Slice<'a> {
    pub fn data(&self) -> &[u8] {
        self.data
    }
}
