#![allow(non_snake_case)]
#![allow(dead_code)]

extern crate lua;
extern crate luakit;

use luakit::Luakit;

#[allow(unused)]
fn main() {
    let mut L = Luakit::new();
    unsafe {
        let res = L.run_script(lua::cstr!("require ('aa')"));
        match res {
            Ok(_) => println!("Script executed successfully"),
            Err(e) => println!("Error: {}", e),
        }
    }
}
