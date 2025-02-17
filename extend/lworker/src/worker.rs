#![allow(non_snake_case)]
#![allow(dead_code)]

use luakit::{ LuaBuf, BaseCodec };

// Lua专用编解码器
pub struct WorkerCodec<'a> {
    base: BaseCodec<'a>,
}

impl<'a> WorkerCodec<'a> {
    pub fn new() -> Self {
        Self {
            base: BaseCodec::new(),
        }
    }

    pub fn err(&self) -> &str { self.base.err() }
    pub fn failed(&self) -> bool { self.base.failed() }
    pub fn error(&mut self, err: String) { self.base.error(err) }
    pub fn set_buf(&mut self, buf: &mut LuaBuf) { self.base.set_buf(buf) }
    pub fn set_slice(&mut self, slice: Slice<'a>) { self.base.set_slice(slice) }
    pub fn load_packet(&mut self, data_len: i32) -> i32 { self.base.load_packet(data_len) }
}

impl<'a> Codec for WorkerCodec<'a> {}