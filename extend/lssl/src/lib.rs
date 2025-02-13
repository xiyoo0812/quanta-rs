#![allow(non_snake_case)]

extern crate lua;
extern crate libc;
extern crate luakit;

mod rsa;

use std::vec;
use rsa::LuaRsaKey;
use lua::lua_State;
use base64::prelude::*;
use std::num::NonZeroU32;
use md5::{ Md5, Digest };
use ring::hmac::{self, HMAC_SHA256, HMAC_SHA512, HMAC_SHA1_FOR_LEGACY_USE_ONLY };
use ring::pbkdf2::{self, PBKDF2_HMAC_SHA1, PBKDF2_HMAC_SHA256, PBKDF2_HMAC_SHA512 };
use ring::digest::{Context, SHA1_FOR_LEGACY_USE_ONLY, SHA256, SHA512, SHA1_OUTPUT_LEN, SHA256_OUTPUT_LEN, SHA512_OUTPUT_LEN};

use luakit::{ Luakit, LuaPush, LuaPushFn, LuaPushFnMut };

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>()
}

fn from_hex(hex_str: &[u8]) -> Vec<u8> {
    if hex_str.len() % 2 != 0 {
        return vec![];
    }
    let mut bytes = Vec::with_capacity(hex_str.len() / 2);
    for chunk in hex_str.chunks(2) {
        let high_nibble = match char::from(chunk[0]).to_digit(16) {
            Some(n) => n << 4,
            None => return vec![],
        };
        let low_nibble = match char::from(chunk[1]).to_digit(16) {
            Some(n) => n,
            None => return vec![],
        };
        bytes.push((high_nibble | low_nibble) as u8);
    }
    bytes
}

fn randomkey(size: usize, hex: bool) -> Vec<u8> {
    let size = if size == 0 { 12 } else { size };
    let mut tmp = vec![0u8; size];
    for i in 0..size {
        tmp[i] = rand::random::<u8>();
    }
    if hex {
        to_hex(&tmp).as_bytes().to_vec()
    } else {
        tmp
    }
}

fn md5(input: &[u8], hex: bool) -> Vec<u8> {
    let mut hasher = Md5::new();
    hasher.update(input);
    let bytes = hasher.finalize().to_vec();
    if hex {
        to_hex(&bytes).as_bytes().to_vec()
    } else {
        bytes
    }
}


macro_rules! ssl_sha_impl {
    ($method:ident, $input:expr) => {{
        let mut ctx = Context::new(&$method);
        ctx.update($input);
        ctx.finish().as_ref().to_vec()
    }}
}

macro_rules! ssl_hmac_impl {
    ($method:ident, $key:expr, $val:expr) => {{
        let s_key = hmac::Key::new($method, $key);
        let mut ctx = hmac::Context::with_key(&s_key);
        ctx.update($val);
        ctx.sign().as_ref().to_vec()
    }}
}

macro_rules! ssl_pbkdf2_impl {
    ($method:ident, $secret:expr, $salt:expr, $iter:expr, $len:expr) => {{
        let mut to_store = [0u8; $len];
        pbkdf2::derive($method, NonZeroU32::new($iter).unwrap(), &$salt, $secret, &mut to_store);
        to_store.to_vec()
    }}
}

fn preprocess(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

#[no_mangle]
pub extern "C" fn luaopen_lssl(L: *mut lua_State) -> i32 {
    let mut kit = Luakit::load(L);
    let mut lssl = kit.new_table(Some("ssl"));
    luakit::set_function!(lssl, "md5", md5);
    luakit::set_function!(lssl, "randomkey", randomkey);
    luakit::set_function!(lssl, "hex_decode", from_hex);
    luakit::set_function!(lssl, "hex_encode", |input: &[u8]| to_hex(input));
    luakit::set_function!(lssl, "sha256", |input: &[u8]| ssl_sha_impl!(SHA256, input));
    luakit::set_function!(lssl, "sha512", |input: &[u8]| ssl_sha_impl!(SHA512, input));
    luakit::set_function!(lssl, "sha1", |input: &[u8]| ssl_sha_impl!(SHA1_FOR_LEGACY_USE_ONLY, input));
    luakit::set_function!(lssl, "hmac_sha256", |key: &[u8], val: &[u8]| ssl_hmac_impl!(HMAC_SHA256, key, val));
    luakit::set_function!(lssl, "hmac_sha512", |key: &[u8], val: &[u8]| ssl_hmac_impl!(HMAC_SHA512, key, val));
    luakit::set_function!(lssl, "hmac_sha1", |key: &[u8], val: &[u8]| ssl_hmac_impl!(HMAC_SHA1_FOR_LEGACY_USE_ONLY, key, val));
    luakit::set_function!(lssl, "pbkdf2_sha1", |secret: &[u8], salt: &[u8], iter:u32| {
        ssl_pbkdf2_impl!(PBKDF2_HMAC_SHA1, secret, salt, iter, SHA1_OUTPUT_LEN)
    });
    luakit::set_function!(lssl, "pbkdf2_sha256", |secret: &[u8], salt: &[u8], iter:u32| {
        ssl_pbkdf2_impl!(PBKDF2_HMAC_SHA256, secret, salt, iter, SHA256_OUTPUT_LEN)
    });
    luakit::set_function!(lssl, "pbkdf2_sha512", |secret: &[u8], salt: &[u8], iter:u32| {
        ssl_pbkdf2_impl!(PBKDF2_HMAC_SHA512, secret, salt, iter, SHA512_OUTPUT_LEN)
    });
    luakit::set_function!(lssl, "b64_encode", |input: &[u8]| -> String {
        BASE64_STANDARD.encode(input)
    });
    luakit::set_function!(lssl, "b64_decode", |input: String| -> Vec<u8> {
        BASE64_STANDARD.decode(preprocess(&input)).unwrap()
    });
    luakit::set_function!(lssl, "rsa_key", || Box::new(LuaRsaKey::new()));
    luakit::new_class!(LuaRsaKey, lssl, "RsaKey",
        "set_pubkey", LuaRsaKey::set_pubkey,
        "set_prikey", LuaRsaKey::set_prikey,
        "encrypt", LuaRsaKey::encrypt,
        "decrypt", LuaRsaKey::decrypt,
        "verify", LuaRsaKey::verify,
        "sign", LuaRsaKey::sign
    );
    lssl.native_to_lua(L)
}
