
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
    let chain = sm3::rainbow::RainbowChain::from_index(head, charset, &range, plaintext_lens.as_ref(), 0, 20, 0);
    log::debug!("Generate chain: {:?}", chain);

    let mut target_hash = [0u8; 32];
    hex::decode_to_slice("8cde2576771b76d33ee1bc168c691bbe302dcc2df6b1cee57a79e51d087bb0cf", &mut target_hash).unwrap();

    match chain.find_match(&target_hash, charset, &range, plaintext_lens.as_ref(), 20, 0) {
        Some(result) => {
            let plain = String::from_utf8_lossy(&result);
            log::info!("Found plain text: {:?}", plain)
        },
        None => log::warn!("Failed to find plain text!")
    }
}
