extern crate pkg_config;
extern crate cc;

fn main() {
    let mut build = cc::Build::new();
    build.file("lua/src/onelua.c").define("MAKE_LUA", "1");

    if cfg!(windows) {
        build.define("LUA_USE_WINDOWS", "1");
    }
    if cfg!(unix) {
        build.define("LUA_USE_LINUX", "1");
    }
    if cfg!(macos) {
        build.define("LUA_USE_MACOSX", "1");
    }

    build.include("lua/src").compile("liblua.a");
}
