use crate::*;
use std::num::Wrapping;
use std::convert::TryFrom;
use std::mem::MaybeUninit;
use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};

// reference:
// https://tools.ietf.org/html/draft-oscca-cfrg-sm3-02


fn my_hash_impl(input: &[u8]) -> Bytes {
    let mut output: [u8; 32] = unsafe { MaybeUninit::uninit().assume_init() };

    #[allow(non_snake_case)]
    let mut V: [Wrapping<u32>; 8] = [
        Wrapping(0x7380166f), Wrapping(0x4914b2b9), Wrapping(0x172442d7), Wrapping(0xda8a0600),
        Wrapping(0xa96f30bc), Wrapping(0x163138aa), Wrapping(0xe38dee4d), Wrapping(0xb0fb0e4e)
    ];

    // preprocessing
    let length: u64 = u64::try_from(input.len()).unwrap();

    // 9: 8-byte length + 0x80
    // padding: 80 00 00 00 ... [64-bit length]
    let real_length = (input.len() + 9 + 63) & (!63usize);
    let mut preprocessed: Vec<u8> = Vec::with_capacity(real_length);
    preprocessed.extend_from_slice(input);
    preprocessed.resize(real_length, 0);
    preprocessed[input.len()] = 0x80;
    // write length in big endian
    (&mut preprocessed[real_length - 8..real_length]).write_u64::<BigEndian>(length * 8).unwrap();

    // main loop
    for offset in (0..real_length).step_by(64) {

        // intentionally uninitialized
        let mut w: [Wrapping<u32>; 68] = unsafe { MaybeUninit::uninit().assume_init() };
        let mut w1: [Wrapping<u32>; 64] = unsafe { MaybeUninit::uninit().assume_init() };

        // B_i = W_0 || ... || W_15
        // copy preprocessed to w[0..15]
        for i in 0..16 {
            w[i] = Wrapping((&preprocessed[offset + 4 * i..offset + 4 * i + 4]).read_u32::<BigEndian>().unwrap());
        }

        // 5.3.2.  Message Expansion Function ME
        // extend 16 words to w[16..67]
        for i in 16..68 {
            // W_j = P_1(W_{j - 16} xor W_{j - 9} xor (W_{j - 3} <<< 15)) xor
            // (W_{j - 13} <<< 7) xor W_{ j - 6 }
            let x = w[i - 16] ^ w[i - 9] ^ ((w[i - 3] << 15) | (w[i - 3] >> 17));
            let p_w = x ^ ((x << 15) | (x >> 17)) ^ ((x << 23) | (x >> 9));
            w[i] = p_w ^ ((w[i - 13] << 7) | (w[i - 13] >> 25)) ^ w[i - 6];
        }

        for i in 0..64 {
            // W'_j = W_j xor W_{j + 4}
            w1[i] = w[i] ^ w[i + 4];
        }

        // E_i = W_0 || ... || W_67 || W'_0 || ... || W'_63

        // 5.3.3. Compression Function CF
        let mut a = V[0];
        let mut b = V[1];
        let mut c = V[2];
        let mut d = V[3];
        let mut e = V[4];
        let mut f = V[5];
        let mut g = V[6];
        let mut h = V[7];

        // compression main loop
        for i in 0..64 {
            // 4.2.  Constants T_j
            let tj = Wrapping::<u32>(if i <= 15 { 0x79cc4519 } else { 0x7a879d8a });
            let mut ss1 = ((a << 12) | (a >> 20)) + e + ((tj << (i % 32)) | (tj >> (32 - i % 32)));
            ss1 = (ss1 << 7) | (ss1 >> 25);
            let ss2 = ss1 ^ ((a << 12) | (a >> 20));
            // 4.3. Boolean Functions FF_j and GG_j
            let tt1 = if i <= 15 { a ^ b ^ c } else { (a & b) | (a & c) | (b & c) } + d + ss2 + w1[i as usize];
            let tt2 = if i <= 15 { e ^ f ^ g } else { (e & f) | ((!e) & g) } + h + ss1 + w[i as usize];

            d = c;
            c = (b << 9) | (b >> 23);
            b = a;
            a = tt1;
            h = g;
            g = (f << 19) | (f >> 13);
            f = e;
            e = tt2 ^ ((tt2 << 9) | (tt2 >> 23)) ^ ((tt2 << 17) | (tt2 >> 15)); // P_0(tt_2)
        }

        V[0] ^= a;
        V[1] ^= b;
        V[2] ^= c;
        V[3] ^= d;
        V[4] ^= e;
        V[5] ^= f;
        V[6] ^= g;
        V[7] ^= h;
    }

    // write to results in big endian
    for i in 0..8 {
        (&mut output[4 * i..4 * (i + 1)]).write_u32::<BigEndian>(V[i].0).unwrap();
    }

    Bytes{
        buf: output,
        len: 32
    }
}

pub const HASH: Hash = my_hash_impl;
