use crate::*;

fn my_hash_impl(input: &[u8]) -> Result<Bytes, ()> {
    unimplemented!("soon");
}

pub const HASH: Hash = my_hash_impl;
