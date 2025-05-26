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
use zstd::stream::{ decode_all, encode_all };
use lz4::block::{ compress, decompress, CompressionMode };
use ring::hmac::{ self, HMAC_SHA256, HMAC_SHA512, HMAC_SHA1_FOR_LEGACY_USE_ONLY };
use ring::pbkdf2::{ self, PBKDF2_HMAC_SHA1, PBKDF2_HMAC_SHA256, PBKDF2_HMAC_SHA512 };
use crc::{Crc, CRC_8_LTE, CRC_16_XMODEM, CRC_32_ISO_HDLC, CRC_64_GO_ISO};
use ring::digest:: {Context, SHA1_FOR_LEGACY_USE_ONLY, SHA256, SHA512, SHA1_OUTPUT_LEN, SHA256_OUTPUT_LEN, SHA512_OUTPUT_LEN};

use luakit::{ Luakit, LuaPush, LuaPushFn, LuaPushFnMut };

const HEX_LUT: [u8; 16] = [
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7',
    b'8', b'9', b'A', b'B', b'C', b'D', b'E', b'F',
];

fn to_hex(bytes: &[u8]) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        buffer.push(HEX_LUT[(byte >> 4) as usize]);
        buffer.push(HEX_LUT[(byte & 0x0F) as usize]);
    }
    buffer
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
        to_hex(&tmp)
    } else {
        tmp
    }
}

fn md5(input: &[u8], hex: bool) -> Vec<u8> {
    let mut hasher = Md5::new();
    hasher.update(input);
    let bytes = hasher.finalize().to_vec();
    if hex {
        to_hex(&bytes)
    } else {
        bytes
    }
}

fn preprocess(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
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

macro_rules! ssl_crc_impl {
    ($bit:ident, $val:expr, $algm:expr) => {{
        let crc = Crc::<$bit>::new(&$algm);
        crc.checksum($val)
    }}
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
    luakit::set_function!(lssl, "crc8", |input: &[u8]| ssl_crc_impl!(u8, input, CRC_8_LTE));
    luakit::set_function!(lssl, "crc16", |input: &[u8]| ssl_crc_impl!(u16, input, CRC_16_XMODEM));
    luakit::set_function!(lssl, "crc32", |input: &[u8]| ssl_crc_impl!(u32, input, CRC_32_ISO_HDLC));
    luakit::set_function!(lssl, "crc64", |input: &[u8]| ssl_crc_impl!(u64, input, CRC_64_GO_ISO));
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
    luakit::set_function!(lssl, "lz4_encode", |input: &[u8]| -> Vec<u8> {
        compress(input, Some(CompressionMode::DEFAULT), true).unwrap()
    });
    luakit::set_function!(lssl, "lz4_decode", |input: &[u8]| -> Vec<u8> {
        decompress(&input, None).unwrap()
    });
    luakit::set_function!(lssl, "zstd_encode", |input: &[u8]| -> Vec<u8> {
        encode_all(input, 3).unwrap()
    });
    luakit::set_function!(lssl, "zstd_decode", |input: &[u8]| -> Vec<u8> {
        decode_all(input).unwrap()
    });
    luakit::set_function!(lssl, "xxtea_encode", |input: Vec<u8>, key: String| -> Vec<u8> {
        xxtea::encrypt_raw(&input, &key)
    });
    luakit::set_function!(lssl, "xxtea_decode", |input: Vec<u8>, key: String| -> Vec<u8> {
        xxtea::decrypt_raw(&input, &key)
    });
    luakit::set_function!(lssl, "lxor_byte", |s1:&[u8], s2:&[u8]| -> Vec<u8> {
        s1.iter().zip(s2.iter()).map(|(&a, &b)| a ^ b).collect()
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
