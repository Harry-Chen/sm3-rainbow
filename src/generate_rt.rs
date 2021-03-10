use sm3::*;
use log::*;

fn main() {
    env_logger::init();

    // head: RainbowIndex, charset: &[u8], plaintext_len_range: &Range<usize>, plaintext_lens: &[u64], length: usize, reduction_offset: u64
    let charset = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".as_bytes();
    let range = 0..8usize;
    let mut plaintext_lens = Vec::new();
    // key space (cumulative)
    plaintext_lens.push(0);

    for i in 0..range.end {
        let prefix_sum = *plaintext_lens.last().unwrap();
        plaintext_lens.push(prefix_sum + charset.len().pow((i + 1) as u32) as u64);
    }

    log::info!("Plain text count: {:?}", plaintext_lens);

    log::debug!("Start generating");
    let head = sm3::rainbow::RainbowIndex(12312451);
    let chain = sm3::rainbow::RainbowChain::from_index(head, charset, &range, plaintext_lens.as_ref(), 20, 0);
    log::debug!("Generate chain: {:?}", chain);
}
