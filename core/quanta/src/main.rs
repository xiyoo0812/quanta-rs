#![allow(non_snake_case)]
#![allow(dead_code)]

extern crate lua;
extern crate luakit;

use lua::cstr;
use luakit::Luakit;

// use std::any::Any;
// use std::rc::Rc;
// use std::cell::RefCell;

// // 定义 dynamic_returns 函数，接受一个可变引用到 Vec<Box<dyn Any>>
// fn dynamic_returns(vec: &mut Vec<Box<dyn Any>>) {
//     println!("dynamic_returns: {}", vec.len());

//     // 修改第一个元素为新的整数值
//     *vec[0].downcast_mut::<Rc<RefCell<i32>>>().unwrap().borrow_mut() = 100;
//     *vec[1].downcast_mut::<Rc<RefCell<f64>>>().unwrap().borrow_mut() = 3.14;
//     *vec[2].downcast_mut::<Rc<RefCell<String>>>().unwrap().borrow_mut() = String::from("hello");
// }

// fn main() {
//     // 初始化包含不同类型的 Box<dyn Any> 的 Vec
//     let x_val = Rc::new(RefCell::new(0));
//     let y_val = Rc::new(RefCell::new(0.0));
//     let z_val = Rc::new(RefCell::new(String::from("")));

//     let mut x: Vec<Box<dyn Any>> = vec![
//         Box::new(Rc::clone(&x_val)),
//         Box::new(Rc::clone(&y_val)),
//         Box::new(Rc::clone(&z_val)),
//     ];

//     // 调用 dynamic_returns 函数，传入可变引用
//     dynamic_returns(&mut x);

//     // 打印修改后的 x_val
//     println!("Modified x_val: {}", x_val.borrow());
//     println!("Modified y_val: {}", y_val.borrow());
//     println!("Modified z_val: {}", z_val.borrow());
// }

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

     //   luakit::lua_call_function!()
    }
}
