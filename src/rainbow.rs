use log::*;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::*;
use crate::my_sm3_impl::my_hash_impl_inplace;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct RainbowIndex(pub u64);

unsafe impl Send for RainbowIndex {}
unsafe impl Sync for RainbowIndex {}

impl RainbowIndex {

    // convert index to plain text
    pub fn to_plaintext(&self, charset: &[u8], plaintext_len_range: &Range<usize>, plaintext_lens: &[u64], plaintext: &mut [u8]) -> usize {

        let index = self.0;
        let mut index_x = index;
        let mut plaintext_len= 0;

        // calculate length
        for l in plaintext_len_range.clone().rev() {
            let len_index = l - 1;
            if self.0 >= plaintext_lens[len_index] {
                plaintext_len = l;
                index_x = index - plaintext_lens[len_index];
                break;
            }
        };

        // fill prefix
        for l in 0..plaintext_len {
            let charset_len = charset.len() as u64;
            plaintext[l as usize] = charset[(index_x % charset_len) as usize];
            index_x /= charset_len;
        }

        // return length to plain text
        plaintext_len
    }

    // reduction functions (from hash to index according to pos)
    pub fn from_hash(hash: &[u8], reduction_offset: u64, plaintext_space_total: u64, pos: u32) -> Self {
        // reinterpret hash[0..8] as u64
        let ret = (&hash[0..8]).read_u64::<LittleEndian>().unwrap();
        RainbowIndex((ret + reduction_offset + pos as u64) % plaintext_space_total)
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct RainbowChain {
    head: RainbowIndex,
    tail: RainbowIndex,
    length: usize
}

unsafe impl Send for RainbowChain {}
unsafe impl Sync for RainbowChain {}


impl RainbowChain {

    pub fn from_index(head: RainbowIndex, charset: &[u8], plaintext_len_range: &Range<usize>, plaintext_lens: &[u64], length: usize, reduction_offset: u64) -> Self {

        let mut index = head;
        log::info!("Starting generating chain from index {:#018x}", index.0);

        // buffer for plain text
        let mut plaintext: Vec<u8> = Vec::new();
        let max_len = (plaintext_len_range.end - 1) as usize;
        plaintext.resize(max_len, 0x3f); // fill with '?'

        // buffer for output
        let mut hash = [0u8; 32];
        let total_space = *plaintext_lens.last().unwrap();

        for i in 0..length {
            let len = index.to_plaintext(&charset, &plaintext_len_range, &plaintext_lens, &mut plaintext);
            my_hash_impl_inplace(&plaintext, len as usize, &mut hash);
            index = RainbowIndex::from_hash(&hash, reduction_offset, total_space, i as u32);
            // log each step
            if log_enabled!(log::Level::Debug) {
                let plaintext_char = unsafe { String::from_raw_parts(plaintext.as_mut_ptr(), max_len, max_len) };
                let hash_char = hex::encode(hash);
                log::debug!("Step {}: plain text {}, length {}, hash {}, new index {:#018x}", i, plaintext_char, len, hash_char, index.0);
                std::mem::forget(plaintext_char);
            }
        }

        log::info!("Rainbow chain has tail index {:#018x}", index.0);

        RainbowChain {
            head,
            tail: index,
            length
        }
    }
}
