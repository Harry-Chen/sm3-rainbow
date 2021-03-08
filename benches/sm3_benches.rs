#![feature(test)]

extern crate test;
use test::{Bencher, black_box};


const OPENSSL_SM3: sm3::Hash = sm3::OPENSSL_SM3;
const MY_SM3: sm3::Hash = sm3::MY_SM3;

fn rand_bytes(len: u32) -> Vec<u8> {
    (0..len).map(|_| { rand::random::<u8>() }).collect::<Vec<u8>>()
}

#[bench]
fn benchmark_openssl_sm3(b: &mut Bencher) {
    let random_bytes = rand_bytes(1024);
    let bytes = random_bytes.as_slice();
    b.iter(|| {black_box(OPENSSL_SM3(bytes))});
}

#[bench]
fn benchmark_my_sm3(b: &mut Bencher) {
    let random_bytes = rand_bytes(1024);
    let bytes = random_bytes.as_slice();
    b.iter(|| {black_box(MY_SM3(bytes))});
}
