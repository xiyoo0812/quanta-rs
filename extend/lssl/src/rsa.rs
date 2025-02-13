#![allow(non_snake_case)]
#![allow(dead_code)]

use luakit::LuaGc;

use pem::parse;
use rsa::sha2::Sha256;
use rsa::traits::PublicKeyParts;
use rsa::{RsaPrivateKey, RsaPublicKey, Pkcs1v15Encrypt};
use rsa::signature::{Signer, SignatureEncoding, Verifier};
use rsa::pkcs1v15::{ SigningKey, VerifyingKey, Signature};
use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::pkcs8::DecodePublicKey;
// 计算最大分块大小 (PKCS#1 v1.5填充)
fn rsa_max_block_size(public_key: &RsaPublicKey) -> usize {
    (public_key.size() as usize) - 11 // 2048位密钥返回 245字节
}

pub struct LuaRsaKey {
    pub_key : Option<RsaPublicKey>,
    priv_key : Option<RsaPrivateKey>
}

impl LuaRsaKey {
    pub fn new() -> Self {
        LuaRsaKey { priv_key: None, pub_key: None }
    }

    pub fn set_pubkey(&mut self, pkey: String) -> bool {
        let pem = parse(pkey.as_bytes()).unwrap();
        match RsaPublicKey::from_public_key_der(pem.contents()) {
            Ok(key) => {
                self.pub_key = Some(key);
                true
            },
            Err(e) => {
                println!("from_pkcs1_pem error: {}", e);
                return false
            }
        }
    }

    pub fn set_prikey(&mut self, pkey: String) -> bool {
        let pem = parse(pkey.as_bytes()).unwrap();
        match RsaPrivateKey::from_pkcs1_der(pem.contents()) {
            Ok(key) => {
                self.pub_key = Some(RsaPublicKey::from(key.clone()));
                self.priv_key = Some(key);
                true
            },
            Err(e) => {
                println!("from_pkcs1_pem error: {}", e);
                return false
            }
        }
    }

    pub fn encrypt(&mut self, bytes: &[u8]) -> Vec<u8> {
        let mut encrypted = Vec::new();
        if let Some(key) = &self.pub_key {
            let mut rng = rand::thread_rng();
            let max_block_size = rsa_max_block_size(key);
            for chunk in bytes.chunks(max_block_size) {
                let chunk = key.encrypt(&mut rng, Pkcs1v15Encrypt, chunk).unwrap();
                encrypted.extend(chunk);
            }
        }
        encrypted
    }

    pub fn decrypt(&self, bytes: &[u8]) -> Vec<u8> {
        let mut decrypted = Vec::new();
        if let Some(key) = &self.priv_key {
            let block_size = key.size() as usize;
            for chunk in bytes.chunks(block_size) {
                let chunk = key.decrypt(Pkcs1v15Encrypt, chunk).unwrap();
                decrypted.extend(chunk);
            }
        }
        decrypted
    }

    pub fn verify(&self, value: &[u8], sign: &[u8]) -> bool {
        if let Some(key) = &self.pub_key {
            let verifying_key = VerifyingKey::<Sha256>::new(key.clone());
            let signature = Signature::try_from(sign).unwrap();
            return verifying_key.verify(value, &signature).is_ok();
        }
        false
    }

    pub fn sign(&self, value: &[u8]) -> Vec<u8> {
        if let Some(key) = &self.priv_key {
            let skey = SigningKey::<Sha256>::new(key.clone());
            return skey.sign(value).to_bytes().as_ref().to_vec();
        }
        vec![]
    }
    
}

impl  LuaGc for LuaRsaKey {}
