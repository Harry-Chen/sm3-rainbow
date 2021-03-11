use itertools::Itertools;
use indicatif::{ProgressBar, ProgressStyle};
use log::*;
use rayon::prelude::*;
use sm3::rainbow::{RainbowChain, RainbowIndex};

fn main() {
    env_logger::builder().init();

    // head: RainbowIndex, charset: &[u8], plaintext_len_range: &Range<usize>, plaintext_lens: &[u64], length: usize, reduction_offset: u64
    let charset = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".as_bytes();
    let range = 5..7usize;
    let mut plaintext_lens = Vec::new();
    // key space (cumulative)
    plaintext_lens.push(0);

    for i in 0..range.end {
        let prefix_sum = *plaintext_lens.last().unwrap();
        plaintext_lens.push(
            prefix_sum
                + if range.start <= i + 1 {
                    charset.len().pow((i + 1) as u32) as u64
                } else {
                    0
                },
        );
    }

    info!("Plain text count: {:?}", plaintext_lens);

    info!("Start generating rainbow chains");
    let rainbow_count = 10000;
    let rainbow_chain_len = 10000;
    let progress = ProgressBar::new(rainbow_count);
    progress.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed}/{eta}] [{bar:50.cyan/blue}] {pos}/{len} ({percent}%)",
            )
            .progress_chars("#>-"),
    );

    let mut chains: Vec<_> = (0..rainbow_count)
        .into_par_iter()
        .map(|i| {
            let head = sm3::rainbow::RainbowIndex(plaintext_lens[4] + i);
            let chain = sm3::rainbow::RainbowChain::from_index(
                head,
                charset,
                &range,
                plaintext_lens.as_ref(),
                0,
                rainbow_chain_len,
                0,
            );
            trace!("Generate chain: {:?}\n", chain);
            progress.inc(1);
            chain
        })
        .collect();

    progress.finish_and_clear();
    info!("Finish generating rainbow chains");

    info!("Start sorting rainbow chains");
    chains.sort();
    info!("Finish sorting rainbow chains");

    // make unique by removing chains with same tails
    chains = chains.into_iter().unique().collect();
    info!("Table size after removing duplicated tails: {}", chains.len());

    let mut target_hash = [0u8; 32];
    hex::decode_to_slice(
        "af67760fb0b62ca056e207c226b9f3e5bec5ad406658b6829ca09b1282e290ca",
        &mut target_hash,
    )
    .unwrap();

    progress.reset();
    progress.set_length(rainbow_chain_len as u64);

    // store cracked plain text
    let cracked: Vec<_> = (0..rainbow_chain_len)
        .into_par_iter()
        .map(|i| {
            progress.inc(1);
            // offset on chain
            let chain_offset = rainbow_chain_len - 1 - i;
            // first step: R_offset
            let mut target_tail = RainbowIndex::from_hash(
                &target_hash,
                0,
                *plaintext_lens.last().unwrap(),
                chain_offset as u32,
            );
            // remaining steps: H, R_{o+1}, H, ..., R_{l-1}
            if i > 0 {
                target_tail = RainbowIndex::traverse_chain(
                    target_tail,
                    charset,
                    &range,
                    &plaintext_lens,
                    chain_offset + 1,
                    i,
                    0,
                    |_, _, _| false,
                );
            }
            debug!(
                "Searching for step {} with target tail {:#018x}\n",
                i, target_tail.0
            );

            let result = match chains.binary_search(&RainbowChain {
                head: RainbowIndex(0),
                tail: target_tail,
            }) {
                Ok(match_idx) => {
                    let match_chain = &chains[match_idx];
                    info!(
                        "Found matching chain {} on step {}: {:?}\n",
                        match_idx, i, match_chain
                    );
                    match match_chain.find_match(
                        &target_hash,
                        charset,
                        &range,
                        plaintext_lens.as_ref(),
                        rainbow_chain_len,
                        0,
                    ) {
                        Some(result) => {
                            let plain = String::from_utf8_lossy(&result).into_owned();
                            info!("Found plain text: {:?}\n", plain);
                            Some(plain)
                        }
                        None => None, // warn!("Failed to find plain text!")
                    }
                }
                Err(_) => {
                    debug!("Not found for step {}\n", i);
                    None
                }
            };

            result
        })
        .filter(|r| !r.is_none())
        .map(|r| r.unwrap())
        .collect();

    progress.finish_and_clear();

    if cracked.is_empty() {
        warn!("No plain text found");
    } else {
        info!("Plain text: {:?}", cracked);
    }
}
