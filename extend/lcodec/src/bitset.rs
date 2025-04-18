#![allow(non_snake_case)]
#![allow(dead_code)]

use luakit::LuaGc;

const MAX_BITSET_SIZE: usize = 1024;
static BITHEX: &[u8] = b"0123456789ABCDEF";

pub struct BitSet {
    m_bits: Vec<bool>,
}

impl LuaGc for BitSet {}

impl BitSet {
    pub fn new() -> Self {
        BitSet { m_bits: vec![false; 16] }
    }

    fn fromhex(&self, x: u8) -> u8 {
        if (b'A'..=b'Z').contains(&x) {
            x - b'A' + 10
        } else if (b'a'..=b'z').contains(&x) {
            x - b'a' + 10
        } else if (b'0'..=b'9').contains(&x) {
            x - b'0'
        } else {
            x
        }
    }

    pub fn load(&mut self, val: &[u8]) -> bool {
        let vsz = val.len();
        if vsz == 0 || vsz * 8 > MAX_BITSET_SIZE {
            return false;
        }
        self.m_bits.resize((vsz + 7) / 8 * 8, false);
        for i in 0..vsz {
            self.m_bits[i] = val[vsz - i - 1] == b'1';
        }
        true
    }

    pub fn loadhex(&mut self, val: &[u8]) -> bool {
        let vsz = val.len();
        if vsz == 0 || vsz % 2 != 0 || vsz * 4 > MAX_BITSET_SIZE {
            return false;
        }
        self.m_bits.resize(vsz * 4, false);
        for (i, chunk) in val.rchunks(2).enumerate() {
            let hi = self.fromhex(chunk[0]);
            let low = self.fromhex(chunk[1]);
            let byte = (hi << 4) | low;
            for j in 0..8 {
                let flag = 1 << j;
                self.m_bits[i * 8 + j] = (byte & flag) == flag;
            }
        }
        true
    }

    pub fn loadbin(&mut self, val: &[u8]) -> bool {
        let vsz = val.len();
        if vsz * 8 > MAX_BITSET_SIZE {
            return false;
        }
        self.m_bits.resize(vsz * 8, false);
        for (i, &byte) in val.iter().enumerate() {
            for j in 0..8 {
                let flag = 1 << j;
                self.m_bits[i * 8 + j] = (byte & flag) == flag;
            }
        }
        true
    }

    pub fn binary(&self) -> Vec<u8> {
        let vsz = self.m_bits.len();
        let casz = (vsz + 7) / 8;
        let mut bitset_buf = vec![0u8; casz];
        for i in 0..casz {
            let mut byte = 0u8;
            for j in 0..8 {
                if i * 8 + j < vsz && self.m_bits[i * 8 + j] {
                    byte |= 1 << j;
                }
            }
            bitset_buf[i] = byte;
        }
        bitset_buf
    }

    pub fn hex(&self) -> Vec<u8> {
        let vsz = self.m_bits.len();
        let casz = (vsz + 7) / 8;
        let mut bitset_buf = vec![0u8; casz * 2];
        for i in 0..casz {
            let mut byte = 0u8;
            for j in 0..8 {
                let index = casz - i - 1;
                if index * 8 + j < vsz && self.m_bits[index * 8 + j] {
                    byte |= 1 << j;
                }
            }
            bitset_buf[i * 2] = BITHEX[(byte >> 4) as usize];
            bitset_buf[i * 2 + 1] = BITHEX[(byte & 0xf) as usize];
        }
        bitset_buf
    }

    pub fn tostring(&self, prefix: bool) -> Vec<u8> {
        let mut prefix = prefix;
        let mut result = Vec::new();
        for &bit in self.m_bits.iter().rev() {
            if bit { prefix = true; }
            if prefix { result.push(if bit { b'1' } else { b'0'}); }
        }
        result
    }

    pub fn get(&self, pos: usize) -> bool {
        if pos == 0 || pos > self.m_bits.len() {
            false
        } else {
            self.m_bits[pos - 1]
        }
    }

    pub fn set(&mut self, pos: usize, bval: bool) -> bool {
        if pos == 0 || pos > MAX_BITSET_SIZE {
            return false;
        }
        if pos > self.m_bits.len() {
            self.m_bits.resize(((pos + 7) / 8) * 8, false);
        }
        self.m_bits[pos - 1] = bval;
        true
    }

    pub fn flip(&mut self, pos: usize) -> bool {
        if pos == 0 || pos > self.m_bits.len() {
            return false;
        }
        self.m_bits[pos - 1] = !self.m_bits[pos - 1];
        true
    }

    pub fn check(&self, pos: usize) -> bool {
        if pos == 0 || pos > self.m_bits.len() {
            return false;
        }
        self.m_bits[..pos].iter().all(|&x| x)
    }

    pub fn reset(&mut self, pos: usize) {
        if pos == 0 {
            self.m_bits = vec![false; 16];
        } else if pos <= self.m_bits.len() {
            self.m_bits[pos - 1] = false;
        }
    }
}
