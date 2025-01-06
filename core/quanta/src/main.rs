#![allow(non_snake_case)]
#![allow(dead_code)]

extern crate lua;
extern crate libc;
extern crate luakit;

mod quanta;

use lua::cstr;
use quanta::Quanta;
use libc::{ setlocale, LC_ALL };

#[allow(unused)]
fn main() {
    //stdout 中文支持
    unsafe { setlocale(LC_ALL, cstr!("en_US.UTF-8")) };
    //初始化引擎
    let mut quanta = Quanta::new();
    if !quanta.init() {
        println!("init failed");
        return;
    }
    //测试
    quanta.test();
    //运行
    quanta.run();
}
