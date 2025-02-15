#![allow(non_snake_case)]
#![allow(dead_code)]

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

    pub fn erase(&mut self, erase_len: usize) -> Option<&[u8]> {
        let new_pos = self.pos + erase_len;
        (new_pos <= self.data.len()).then(|| {
            let slice = &self.data[self.pos..new_pos];
            self.pos = new_pos;
            slice
        })
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

    pub fn contents(&self) -> &[u8] {
        &self.data[self.pos..]
    }

    pub fn remaining(&self) -> &[u8] {
        &self.data[self.pos..]
    }
}

