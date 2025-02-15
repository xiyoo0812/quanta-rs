extern crate cc;

fn main() {
    let mut build = cc::Build::new();
    build.file("pb/pb.c");
    build.include("pb").include("../lua/lua").compile("libpb.a");
}
