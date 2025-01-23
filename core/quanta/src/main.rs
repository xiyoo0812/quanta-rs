#![allow(non_snake_case)]
#![allow(dead_code)]

extern crate lua;
extern crate libc;
extern crate luakit;
extern crate signal_hook;

mod test;
mod quanta;

use std::env;
use lua::cstr;
use quanta::Quanta;
use libc::{ setlocale, LC_ALL };


fn main() {
    //stdout 中文支持
    unsafe { setlocale(LC_ALL, cstr!("en_US.UTF-8")) };
    //获取启动参数
    let args: Vec<String> = env::args().collect();
    //初始化引擎
    let mut quanta = Quanta::new();
    if !quanta.setup(args) {
        println!("init failed");
        return;
    }
    //运行
    quanta.run();
}
