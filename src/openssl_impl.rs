use crate::*;
use openssl::hash::*;


fn openssl_hash_impl(input: &[u8]) -> Result<Bytes, ()> {
    let digest = MessageDigest::from_name("sm3").unwrap();
    let hash_result: &[u8] = &openssl::hash::hash(digest, input).unwrap();
    let mut buf: [u8; 32] = Default::default();
    buf.copy_from_slice(&hash_result[0..32]);
    Ok(Bytes{
        buf: buf,
        len: hash_result.len()
    })
}

pub const HASH: Hash = openssl_hash_impl;
