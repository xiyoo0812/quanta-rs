#![allow(non_snake_case)]
#![allow(dead_code)]
use std::ops::{Deref, DerefMut};

use lua::lua_State;
use luakit::{ Codec, BaseCodec, CodecError };

pub struct WorkerCodec<'a> {
    base: BaseCodec<'a>,
}

impl<'a> WorkerCodec<'a> {
    pub fn new() -> Self {
        Self { base: BaseCodec::new() }
    }
}

impl<'a> Deref for WorkerCodec<'a> {
    type Target = BaseCodec<'a>;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl<'a> DerefMut for WorkerCodec<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl<'a> Codec for WorkerCodec<'a> {
    fn decode(&mut self, L: *mut lua_State) -> Result<i32, CodecError>{
        self.base.decode_impl(L)
    }
}