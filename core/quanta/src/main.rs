#![allow(non_snake_case)]
#![allow(dead_code)]

extern crate lua;
extern crate libc;
extern crate luakit;

use lua::cstr;
use luakit::Luakit;
use libc::{ setlocale, LC_ALL };

#[allow(unused)]
fn main() {
    //stdout 中文支持
    unsafe { setlocale(LC_ALL, cstr!("en_US.UTF-8")) };

    let mut L = Luakit::new();
    let res = L.run_file(cstr!("aa.lua"));
    match res {
        Ok(_) => println!("run_script executed successfully"),
        Err(e) => println!("Error: {}", e),
    }

    let res = L.call_function(cstr!("test"));
    match res {
        Ok(_) => println!("call_function executed successfully"),
        Err(e) => println!("Error: {}", e),
    }
    let ret = L.call_lua2(cstr!("test1"), 3, 1, 2);
    match ret {
        Ok(mut refs) => {
            let r1 = refs[0].get::<i32>().unwrap();
            let r2 = refs[1].get::<i32>().unwrap();
            let r3 = refs[2].get::<String>().unwrap();
            println!("call_lua1 ret {}: {}, {}", r1, r2, r3);
        }
        Err(e) => println!("Error: {}", e),
    }
}
