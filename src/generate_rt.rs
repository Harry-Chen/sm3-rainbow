use std::fs::OpenOptions;
use std::path::Path;
use indicatif::{ProgressBar, ProgressStyle};
use log::*;
use rayon::prelude::*;
use sm3::rainbow::{RainbowChain, RainbowIndex};
use clap::Clap;

mod args;
use args::CommonOptions;
use std::io::Write;

fn main() {

    // init program
    env_logger::builder().init();
    let table_opts: CommonOptions = CommonOptions::parse();
    println!("Program options: {:?}", table_opts);

    // read options
    let charset: &[u8] = table_opts.charset.as_bytes();
    let range = (table_opts.min_length as usize)..(table_opts.max_length + 1) as usize;
    let mut plaintext_lens = Vec::new();
    let rainbow_count = table_opts.num_chain as u64;
    let rainbow_chain_len = table_opts.chain_len as usize;
    let table_index = table_opts.table_index as usize;
    let output_file = match table_opts.output_file {
        Some(file) => file,
        None => {
            format!("sm3_m{}_M{}_l{}_c{}_i{:04}.dat", table_opts.min_length, table_opts.max_length, rainbow_chain_len, rainbow_count, table_index)
        }
    };

    println!("Using {} as output file", output_file);
    if Path::exists(Path::new(&output_file)) {
        error!("File already exists");
        std::process::exit(1);
    }
    let mut output = OpenOptions::new().read(true).write(true).create(true).open(&output_file).expect("Cannot open output file");

    // calculate key space (cumulative)
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

    // show progress bar
    let progress = ProgressBar::new(rainbow_count);
    progress.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed}/{eta}] [{bar:50.cyan/blue}] {pos}/{len} ({percent}%)",
            )
            .progress_chars("#>-"),
    );

    // generate chain in parallel
    info!("Start generating rainbow chains");

    let mut chains: Vec<_> = (0..rainbow_count)
        .into_par_iter()
        .map(|i| {
            let head = RainbowIndex(i);
            let chain = RainbowChain::from_index(
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

    // process progress bar
    info!("Start sorting rainbow chains");
    chains.sort();
    chains.dedup();
    info!("Finish sorting rainbow chains");
    info!("Table size after removing duplicated tails: {}", chains.len());

    // write rainbow tables to file
    match output.write(unsafe {
        std::slice::from_raw_parts(
        chains.as_ptr() as *const u8,
        chains.len() * std::mem::size_of::<RainbowChain>(),
    )}
    ) {
        Ok(len) => {
            info!("Successfully writing {} bytes to {}", len, &output_file);
        },
        Err(err) => {
            error!("Error writing file: {:?}", err);
            std::process::exit(2);
        }
    }

    // let mut target_hash = [0u8; 32];
    // hex::decode_to_slice(
    //     "af67760fb0b62ca056e207c226b9f3e5bec5ad406658b6829ca09b1282e290ca",
    //     &mut target_hash,
    // )
    // .unwrap();
    //
    // progress.reset();
    // progress.set_length(rainbow_chain_len as u64);
    //
    // // store cracked plain text
    // let cracked: Vec<_> = (0..rainbow_chain_len)
    //     .into_par_iter()
    //     .map(|i| {
    //         progress.inc(1);
    //         // offset on chain
    //         let chain_offset = rainbow_chain_len - 1 - i;
    //         // first step: R_offset
    //         let mut target_tail = RainbowIndex::from_hash(
    //             &target_hash,
    //             0,
    //             *plaintext_lens.last().unwrap(),
    //             chain_offset as u32,
    //         );
    //         // remaining steps: H, R_{o+1}, H, ..., R_{l-1}
    //         if i > 0 {
    //             target_tail = RainbowIndex::traverse_chain(
    //                 target_tail,
    //                 charset,
    //                 &range,
    //                 &plaintext_lens,
    //                 chain_offset + 1,
    //                 i,
    //                 0,
    //                 |_, _, _| false,
    //             );
    //         }
    //         debug!(
    //             "Searching for step {} with target tail {:#018x}\n",
    //             i, target_tail.0
    //         );
    //
    //         let result = match chains.binary_search(&RainbowChain {
    //             head: RainbowIndex(0),
    //             tail: target_tail,
    //         }) {
    //             Ok(match_idx) => {
    //                 let match_chain = &chains[match_idx];
    //                 info!(
    //                     "Found matching chain {} on step {}: {:?}\n",
    //                     match_idx, i, match_chain
    //                 );
    //                 match match_chain.find_match(
    //                     &target_hash,
    //                     charset,
    //                     &range,
    //                     plaintext_lens.as_ref(),
    //                     rainbow_chain_len,
    //                     0,
    //                 ) {
    //                     Some(result) => {
    //                         let plain = String::from_utf8_lossy(&result).into_owned();
    //                         info!("Found plain text: {:?}\n", plain);
    //                         Some(plain)
    //                     }
    //                     None => None, // warn!("Failed to find plain text!")
    //                 }
    //             }
    //             Err(_) => {
    //                 debug!("Not found for step {}\n", i);
    //                 None
    //             }
    //         };
    //
    //         result
    //     })
    //     .filter(|r| !r.is_none())
    //     .map(|r| r.unwrap())
    //     .collect();
    //
    // progress.finish_and_clear();
    //
    // if cracked.is_empty() {
    //     warn!("No plain text found");
    // } else {
    //     info!("Plain text: {:?}", cracked);
    // }
}
