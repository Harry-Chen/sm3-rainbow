use rayon::prelude::*;
use log::*;

fn main() {

    env_logger::builder().init();

    // head: RainbowIndex, charset: &[u8], plaintext_len_range: &Range<usize>, plaintext_lens: &[u64], length: usize, reduction_offset: u64
    let charset = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".as_bytes();
    let range = 0..7usize;
    let mut plaintext_lens = Vec::new();
    // key space (cumulative)
    plaintext_lens.push(0);

    for i in 0..range.end {
        let prefix_sum = *plaintext_lens.last().unwrap();
        plaintext_lens.push(prefix_sum + charset.len().pow((i + 1) as u32) as u64);
    }

    info!("Plain text count: {:?}", plaintext_lens);

    info!("Start generating rainbow chains");
    let chains: Vec<_> = (0..2u64).into_par_iter().map(|i| {
        let head = sm3::rainbow::RainbowIndex(plaintext_lens[4] + i);
        let chain = sm3::rainbow::RainbowChain::from_index(head, charset, &range, plaintext_lens.as_ref(), 0, 20, 0);
        trace!("Generate chain: {:?}", chain);
        chain
    }).collect();
    // let head = sm3::rainbow::RainbowIndex(plaintext_lens[4]);
    // let chain = sm3::rainbow::RainbowChain::from_index(head, charset, &range, plaintext_lens.as_ref(), 0, 200, 0);

    let mut target_hash = [0u8; 32];
    hex::decode_to_slice("c35fbbd3346482874d22cdfc66500585f7dd1acb67ceb042c54195007a64f0e6", &mut target_hash).unwrap();

    for chain in chains.iter() {
        match chain.find_match(&target_hash, charset, &range, plaintext_lens.as_ref(), 50, 0) {
            Some(result) => {
                let plain = String::from_utf8_lossy(&result);
                info!("Found plain text: {:?}", plain)
            },
            None => error!("Failed to find plain text!")
        }
    }
}
