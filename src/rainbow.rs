use byteorder::{LittleEndian, ReadBytesExt};
use log::*;

use crate::my_sm3_impl::my_hash_impl_inplace;
use crate::*;

#[repr(C)]
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct RainbowIndex(pub u64);

unsafe impl Send for RainbowIndex {}
unsafe impl Sync for RainbowIndex {}

impl RainbowIndex {
    // convert index to plain text
    pub fn to_plaintext(
        &self,
        charset: &[u8],
        plaintext_len_range: &Range<usize>,
        plaintext_lens: &[u64],
        plaintext: &mut [u8],
    ) -> usize {
        let index = self.0;
        let mut index_x = index;
        let mut plaintext_len = 0;

        // calculate length
        for l in plaintext_len_range.clone().rev() {
            let len_index = l - 1;
            if self.0 >= plaintext_lens[len_index] {
                plaintext_len = l;
                index_x = index - plaintext_lens[len_index];
                break;
            }
        }

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
    pub fn from_hash(
        hash: &[u8],
        reduction_offset: u64,
        plaintext_space_total: u64,
        pos: u32,
    ) -> Self {
        // reinterpret hash[0..8] as u64
        let ret = (&hash[0..8]).read_u64::<LittleEndian>().unwrap();
        RainbowIndex((ret + reduction_offset + pos as u64) % plaintext_space_total)
    }

    // traverse the chain from certain position, return the tail index
    pub fn traverse_chain<F>(
        head: RainbowIndex,
        charset: &[u8],
        plaintext_len_range: &Range<usize>,
        plaintext_lens: &[u64],
        start_pos: usize,
        length: usize,
        reduction_offset: u64,
        mut callback: F,
    ) -> Self
    where
        F: FnMut(&[u8], &[u8], usize) -> bool,
    {
        let mut index = head;
        debug!(
            "Starting traversing chain from index {:#018x} with start pos {} length {}",
            index.0, start_pos, length
        );

        // buffer for plain text
        let mut plaintext: Vec<u8> = Vec::new();
        let max_len = (plaintext_len_range.end - 1) as usize;
        plaintext.resize(max_len, 0x3f); // fill with '?'

        // buffer for output
        let mut hash = [0u8; 32];
        let total_space = *plaintext_lens.last().unwrap();

        for pos in start_pos..start_pos + length {
            let len = index.to_plaintext(
                &charset,
                &plaintext_len_range,
                &plaintext_lens,
                &mut plaintext,
            );
            my_hash_impl_inplace(&plaintext, len as usize, &mut hash);
            index = RainbowIndex::from_hash(&hash, reduction_offset, total_space, pos as u32);
            // log each step
            if log_enabled!(log::Level::Debug) {
                let plaintext_char =
                    unsafe { String::from_raw_parts(plaintext.as_mut_ptr(), max_len, max_len) };
                let hash_char = hex::encode(hash);
                trace!(
                    "Pos {}: plain text {}, length {}, hash {}, new index {:#018x}",
                    pos,
                    plaintext_char,
                    len,
                    hash_char,
                    index.0
                );
                std::mem::forget(plaintext_char);
            }
            // invoke callback
            if callback(&hash, &plaintext, len) {
                info!("Traversing stopped by callback at step {}", pos);
                break;
            }
        }

        debug!("Rainbow chain has tail index {:#018x}", index.0);
        index
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Eq)]
// a chain in the rainbow table
pub struct RainbowChain {
    pub head: RainbowIndex,
    pub tail: RainbowIndex,
}

impl PartialEq for RainbowChain {
    fn eq(&self, other: &Self) -> bool {
        self.tail == other.tail
    }
}

impl Ord for RainbowChain {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.tail.cmp(&other.tail)
    }
}

impl PartialOrd for RainbowChain {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

unsafe impl Send for RainbowChain {}
unsafe impl Sync for RainbowChain {}

impl RainbowChain {
    // generate a chain from index as head
    pub fn from_index(
        head: RainbowIndex,
        charset: &[u8],
        plaintext_len_range: &Range<usize>,
        plaintext_lens: &[u64],
        start_pos: usize,
        length: usize,
        reduction_offset: u64,
    ) -> Self {
        let tail = RainbowIndex::traverse_chain(
            head,
            charset,
            plaintext_len_range,
            plaintext_lens,
            start_pos,
            length,
            reduction_offset,
            |_, _, _| false,
        );
        RainbowChain { head, tail }
    }

    // find exact match from head
    pub fn find_match(
        &self,
        target_hash: &[u8],
        charset: &[u8],
        plaintext_len_range: &Range<usize>,
        plaintext_lens: &[u64],
        length: usize,
        reduction_offset: u64,
    ) -> Option<Vec<u8>> {
        if log_enabled!(log::Level::Info) {
            let hash_str = hex::encode(target_hash);
            debug!("Finding match for {} on chain {:?}", hash_str, self);
        }
        // buffer for result
        let mut result: Vec<u8> = Vec::new();
        // check hash in each loop
        RainbowIndex::traverse_chain(
            self.head,
            charset,
            plaintext_len_range,
            plaintext_lens,
            0,
            length,
            reduction_offset,
            |hash, text, len| {
                if target_hash == hash {
                    result.extend_from_slice(&text[..len]);
                    true
                } else {
                    false
                }
            },
        );
        // return result
        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }
}
