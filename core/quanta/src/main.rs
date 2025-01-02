#![allow(non_snake_case)]
#![allow(dead_code)]

extern crate lua;
extern crate luakit;

use lua::cstr;
use luakit::Luakit;

#[allow(unused)]
fn main() {
    let mut L = Luakit::new();
    unsafe {
        let res = L.run_script(cstr!("require ('aa')"));
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
                let r1 : i32 = refs[0].get_i32().unwrap();
                let r2 : i32 = refs[1].get_i32().unwrap();
                let r3 : String = refs[2].get_string().unwrap();
                println!("call_lua1 ret {}: {}, {}", r1, r2, r3);
            }
            Err(e) => println!("Error: {}", e),
        }
    }
}
