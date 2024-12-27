#![allow(non_snake_case)]
#![allow(dead_code)]

use std::collections::HashMap;

#[allow(unused)]
fn main() {
    // Hello
    println!("Hello, world!");

    // 数字类型
    let mut i = 5;
    i += 3;
    let f: f32 = 42.0;

    // 字符串类型
    let s = "SomeString";
    let t = "SomeOtherString";
    let mut u: String = "The".to_string();
    u.push_str("ThirdString");

    // Vec; 运行很好！
    let nums = vec![1, 2, 3, 4, 5];

    // HashMap; 运行效果不佳 :(
    let mut map = HashMap::<String, String>::new();
    map.insert("some_key".to_string(), "some_value".to_string());
    map.insert("some_other_key".to_string(), "some_other_value".to_string());

    // Goodbye
    println!("Goodbye cruel world");
}