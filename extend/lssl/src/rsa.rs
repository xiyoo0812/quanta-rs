#![allow(non_snake_case)]
#![allow(dead_code)]

use luakit::LuaGc;

use rsa::sha2::Sha256;
use base64::prelude::*;
use rsa::signature::Signer;
use rsa::pkcs1v15::SigningKey;
use rsa::{RsaPrivateKey, RsaPublicKey, Pkcs1v15Encrypt};
use rsa::pkcs1::{DecodeRsaPrivateKey, DecodeRsaPublicKey};
use rsa::signature::{Keypair, SignatureEncoding, Verifier};

pub struct LuaRsaKey {
    pub_key : Option<RsaPublicKey>,
    priv_key : Option<RsaPrivateKey>
}

impl LuaRsaKey {
    pub fn new() -> Self {
        LuaRsaKey { priv_key: None, pub_key: None }
    }

    pub fn init_pubkey(&mut self, pkey: String) -> bool {
        let output = BASE64_STANDARD.decode(pkey);
        match output {
            Err(e) => {
                println!("base64 decode error: {}", e);
                return false;
            },
            Ok(output) => { 
                match RsaPublicKey::from_pkcs1_der(&output) {
                    Ok(key) => {
                        self.pub_key = Some(key);
                        true
                    },
                    Err(_) => false
                }
            }
        }
    }

    pub fn init_prikey(&mut self, pkey: String) -> bool {
        let output = BASE64_STANDARD.decode(pkey);
        match output {
            Err(e) => {
                println!("base64 decode error: {}", e);
                return false;
            },
            Ok(output) => { 
                match RsaPrivateKey::from_pkcs1_der(&output) {
                    Ok(key) => {
                        self.pub_key = Some(RsaPublicKey::from(key.clone()));
                        self.priv_key = Some(key);
                        true
                    },
                    Err(_) => false
                }
            }
        }
    }

    pub fn encrypt(&mut self, value: &[u8]) -> Vec<u8> {
        if let Some(key) = &self.pub_key {
            let mut rng = rand::thread_rng();
            return key.encrypt(&mut rng, Pkcs1v15Encrypt, value).unwrap()
        }
        vec![]
    }

    pub fn verify(&self, value: &[u8]) -> bool {
        if let Some(key) = &self.priv_key {
            let skey = SigningKey::<Sha256>::new(key.clone());
            let verifying_key = skey.verifying_key();
            let signature = skey.sign(value);
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
    
    pub fn decrypt(&self, value: &[u8]) -> Vec<u8> {
        if let Some(key) = &self.priv_key {
            return key.decrypt(Pkcs1v15Encrypt, value).unwrap();
        }
        vec![]
    }
}

impl  LuaGc for LuaRsaKey {}
