extern crate cc;

fn main() {
    let mut build = cc::Build::new();
    build.file("lua/onelua.c").define("MAKE_LIB", "1");

    if cfg!(windows) {
        build.define("LUA_USE_WINDOWS", "1");
    }
    if cfg!(unix) {
        build.define("LUA_USE_LINUX", "1");
    }
    if cfg!(target_os = "macos") {
        build.define("LUA_USE_MACOSX", "1");
    }

    build.include("lua").compile("liblua.a");
}
