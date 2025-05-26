// build.rs
fn main() {
    // 获取当前构建模式（debug/release）
    let profile = std::env::var("PROFILE").unwrap();
    println!("{}", profile);
    // 根据构建模式拼接路径
    let lib_path = format!("./target/{}/deps", profile);
    println!("{}", lib_path);
    // 传递链接库搜索路径给编译器
    println!("cargo:rustc-link-search=native={}", lib_path);
    if cfg!(windows) {
        println!("cargo:rustc-link-lib=dylib=lualog.dll");
    } else {
        println!("cargo:rustc-link-lib=dylib=lualog.so");
    }
}