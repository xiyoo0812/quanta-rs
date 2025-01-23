#![allow(non_snake_case)]
#![allow(dead_code)]

extern crate lua;
extern crate libc;
extern crate luakit;

use lua::{cstr, lua_State};
use luakit::{ LuaGc, LuaPush, Luakit, LuaPushFn, LuaPushLuaFn, LuaPushFnMut, LuaPushLuaFnMut };

pub struct Test {
    pub i: i32,
}

pub struct Test2 {
    pub i: i32,
    pub j: i32,
}

impl Test {
    fn new() -> Self {
        Test { i : 2}
    }

    fn some_method1(&self, L: *mut lua_State) -> i32 {
        let j = lua::lua_tointeger(L,1) as i32;
        j.native_to_lua(L)
    }

    fn some_method2(&mut self, L: *mut lua_State) -> i32 {
        let j = lua::lua_tointeger(L,1) as i32;
        let mut kit = Luakit::load(L);
        let res = luakit::object_call!(kit, self, "test", 0, j);
        match res {
            Ok(_) => println!("object_call test executed successfully"),
            Err(e) => println!("Error: {}", e),
        }
        self.i += j;
        self.i.native_to_lua(L)
    }

    fn some_method3(&self, i: i32) -> i32 {
        i
    }

    fn some_method4(&mut self, i: i32) -> i32 {
        self.i += i;
        self.i
    }
}

impl  LuaGc for Test {}

impl Test2 {
    fn new() -> Self {
        Test2 { i : 2, j:0}
    }

    fn some_method_1(&self, L: *mut lua_State) -> i32 {
        let j = lua::lua_tointeger(L,1) as i32;
        j.native_to_lua(L)
    }

    fn some_method_2(&mut self, L: *mut lua_State) -> i32 {
        let j = lua::lua_tointeger(L,1) as i32;
        self.i += j;
        self.i.native_to_lua(L)
    }

    fn some_method_3(&self, i: i32) -> i32 {
        i
    }

    fn some_method_4(&mut self, i: i32) -> i32 {
        self.i += i;
        self.i
    }
}

impl LuaGc for Test2 {}

impl Drop for Test {
    fn drop(&mut self) {
        println!("Test::drop");
    }
}

impl Drop for Test2 {
    fn drop(&mut self) {
        println!("Test2::drop");
    }
}

pub fn luakit_test(kit: &mut Luakit) {
    luakit::new_class!(Test, kit, "Test",
        "some_method1", Test::some_method1,
        "some_method2", Test::some_method2,
        "some_method3", Test::some_method3,
        "some_method4", Test::some_method4
    );
    luakit::new_class!(Test2, kit, "Test2",
        "some_method_1", Test2::some_method_1,
        "some_method_2", Test2::some_method_2,
        "some_method_3", Test2::some_method_3,
        "some_method_4", Test2::some_method_4;
        "i" => i: i32
    );
    let mut luit = kit.new_table(Some("luakit"));
    luakit::set_function!(luit, "test", || { Some(Test::new()) });
    luakit::set_function!(luit, "test2", || { Some(Test2::new()) });

    luakit::set_function!(kit, "wrapper", ||{
        println!("func_wrapper"); return 1001;
    });
    luakit::set_function!(kit, "wrapper1", |a : i32|{
        println!("func_wrapper1 {} {}", a, a * 2); return "abcdefg";
    });
    luakit::set_function!(kit, "wrapper2", |a : i32, b : i32|{
        println!("func_wrapper2 {} {} {}", a, b, a+b); return 1002;
    });

    let res = kit.table_call1("global", "test", 0, 1);
    match res {
        Ok(_) => println!("table_call global.test executed successfully"),
        Err(e) => println!("Error: {}", e),
    }
    let res = kit.call("test");
    match res {
        Ok(_) => println!("call test executed successfully"),
        Err(e) => println!("Error: {}", e),
    }
    let res = kit.call("test2");
    match res {
        Ok(_) => println!("call test2 executed successfully"),
        Err(e) => println!("Error: {}", e),
    }
    let ret = luakit::call!(kit, "test1", 3, 1, 2);
    match ret {
        Ok(mut refs) => {
            let r1 = refs[0].get::<i32>().unwrap();
            let r2 = refs[1].get::<i32>().unwrap();
            let r3 = refs[2].get::<String>().unwrap();
            println!("call test1 ret {}: {}, {}", r1, r2, r3);
        }
        Err(e) => println!("Error: {}", e),
    }
}