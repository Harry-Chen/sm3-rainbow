use std::ops::*;
use std::*;

pub const SM_DIGEST_SIZE: u32 = 32;

#[derive(Copy)]
pub struct Bytes {
    pub(crate) buf: [u8; SM_DIGEST_SIZE as usize],
    pub(crate) len: usize,
}

impl Clone for Bytes {
    #[inline]
    fn clone(&self) -> Bytes {
        *self
    }
}

impl Deref for Bytes {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &[u8] {
        &self.buf[..self.len]
    }
}

impl DerefMut for Bytes {
    #[inline]
    fn deref_mut(&mut self) -> &mut [u8] {
        &mut self.buf[..self.len]
    }
}

impl AsRef<[u8]> for Bytes {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.deref()
    }
}

impl fmt::Debug for Bytes {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, fmt)
    }
}

pub type Hash = fn(input: &[u8]) -> Bytes;

pub mod my_sm3_impl;
pub(crate) mod openssl_sm3_impl;
pub mod rainbow;

pub const OPENSSL_SM3: Hash = openssl_sm3_impl::HASH;
pub const MY_SM3: Hash = my_sm3_impl::HASH;
