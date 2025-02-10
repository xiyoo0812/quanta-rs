#![allow(non_snake_case)]

extern crate lua;
extern crate libc;
extern crate luakit;
use md5::{ Md5, Digest };
use base64ct::{Base64, Encoding};
use rand::Rng;
use ring::digest::{Context, SHA1_FOR_LEGACY_USE_ONLY, SHA256, SHA512};

use lua::lua_State;
use luakit::{ Luakit, LuaPush, LuaPushFn };

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)) .collect::<String>()
}

fn from_hex(hex_str: String) -> Option<Vec<u8>> {
    if hex_str.len() % 2 != 0 {
        return None;
    }
    let mut bytes = Vec::with_capacity(hex_str.len() / 2);
    for chunk in hex_str.as_bytes().chunks(2) {
        let high_nibble = match char::from(chunk[0]).to_digit(16) {
            Some(n) => n << 4,
            None => return None,
        };
        let low_nibble = match char::from(chunk[1]).to_digit(16) {
            Some(n) => n,
            None => return None,
        };
        bytes.push((high_nibble | low_nibble) as u8);
    }
    Some(bytes)
}

fn randomkey(size: usize, hex: bool) -> Option<Vec<u8>> {
    let size = if size == 0 { 12 } else { size };
    let mut tmp = vec![0u8; size];
    let mut rng = rand::rng();
    for i in 0..size {
        tmp[i] = rng.random::<u8>();
    }
    if hex {
        Some(to_hex(&tmp).as_bytes().to_vec())
    } else {
        Some(tmp)
    }
}


#[no_mangle]
pub extern "C" fn luaopen_lssl(L: *mut lua_State) -> i32 {
    let mut kit = Luakit::load(L);
    let mut lssl = kit.new_table(Some("ssl"));
    luakit::set_function!(lssl, "md5", |input : String, hex: bool| -> Option<Vec<u8>> {
        let mut hasher = Md5::new();
        hasher.update(input.as_bytes());
        let bytes = hasher.finalize().to_vec();
        if hex {
            Some(to_hex(&bytes).as_bytes().to_vec())
        } else {
            Some(bytes)
        }
    });
    luakit::set_function!(lssl, "randomkey", randomkey);
    luakit::set_function!(lssl, "hex_decode", from_hex);
    luakit::set_function!(lssl, "hex_encode", |input : Option<Vec<u8>>| -> String {
        to_hex(&input.unwrap())
    });
    luakit::set_function!(lssl, "sha1", |input : String| -> Option<Vec<u8>> {
        let mut ctx = Context::new(&SHA1_FOR_LEGACY_USE_ONLY);
        ctx.update(input.as_bytes());
        Some(ctx.finish().as_ref().to_vec())
    });
    luakit::set_function!(lssl, "sha256", |input : String| -> Option<Vec<u8>> {
        let mut ctx = Context::new(&SHA256);
        ctx.update(input.as_bytes());
        Some(ctx.finish().as_ref().to_vec())
    });
    luakit::set_function!(lssl, "sha512", |input : String| -> Option<Vec<u8>> {
        let mut ctx = Context::new(&SHA512);
        ctx.update(input.as_bytes());
        Some(ctx.finish().as_ref().to_vec())
    });
    luakit::set_function!(lssl, "b64_encode", |input : String| -> String {
        let mut enc_buf = [0u8; 128];
        Base64::encode(input.as_bytes(), &mut enc_buf).unwrap().to_string()
    });
    luakit::set_function!(lssl, "b64_decode", |input : String| -> Option<Vec<u8>> {
        let mut dec_buf = [0u8; 128];
        Some(Base64::decode(input.as_bytes(), &mut dec_buf).unwrap().to_vec())
    });
    lssl.native_to_lua(L)
}
