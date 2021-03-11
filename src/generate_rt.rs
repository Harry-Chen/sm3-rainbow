use std::fs::OpenOptions;
use std::path::Path;
use indicatif::{ProgressBar, ProgressStyle};
use log::*;
use rayon::prelude::*;
use sm3::rainbow::{RainbowChain, RainbowIndex, RainbowTableHeader, RAINBOW_TABLE_HEADER_MAGIC};
use clap::Clap;
use rand::Rng;

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
    let num_chain = table_opts.num_chain;
    let chain_len = table_opts.chain_len;
    let table_index = table_opts.table_index;
    let output_file = match table_opts.output_file {
        Some(file) => file,
        None => {
            format!("sm3_m{}_M{}_l{}_c{}_i{:04}.dat", table_opts.min_length, table_opts.max_length, chain_len, num_chain, table_index)
        }
    };

    println!("Using {} as output file", output_file);
    if Path::exists(Path::new(&output_file)) {
        warn!("File already exists: {}", &output_file);
        if !table_opts.force_overwrite {
            std::process::exit(1);
        }
        warn!("Overwriting {} due to force flag", &output_file);
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
    let plaintext_space_size = plaintext_lens[range.end - 1];
    info!("Plain text count: {:?}, space size: {}", plaintext_lens, plaintext_space_size);

    // show progress bar
    let progress = ProgressBar::new(num_chain);
    progress.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed}/{eta}] [{bar:50.cyan/blue}] {pos}/{len} ({percent}%)",
            )
            .progress_chars("#>-"),
    );

    // generate chain in parallel
    let start_index = table_index * num_chain;
    let end_index = start_index + num_chain;
    info!("Start generating rainbow chains from index {} to {}", start_index, end_index);

    let mut chains: Vec<_> = (start_index..end_index)
        .into_par_iter()
        .map(|i| {
            let head = RainbowIndex(i);
            let chain = RainbowChain::from_index(
                head,
                charset,
                &range,
                plaintext_lens.as_ref(),
                0,
                chain_len as usize,
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

    // generate from random indices until reaching num_chain
    while chains.len() < num_chain as usize {
        let num_remain_chain = (num_chain as usize) - chains.len();
        info!("Generating remaining {} chains from random numbers", num_remain_chain);
        // generate random indices
        let mut rng = rand::thread_rng();
        let remaining_index: Vec<_> = (0..num_remain_chain).map(|_| {
            rng.gen_range(0..plaintext_space_size)
        }).collect();
        // generate random chains
        let mut random_chains: Vec<_> = (0..num_remain_chain)
            .into_par_iter()
            .map(|i| {
                let head = RainbowIndex(remaining_index[i]);
                let chain = RainbowChain::from_index(
                    head,
                    charset,
                    &range,
                    plaintext_lens.as_ref(),
                    0,
                    chain_len as usize,
                    0,
                );
                trace!("Generate chain: {:?}\n", chain);
                progress.inc(1);
                chain
            })
            .collect();
        chains.append(&mut random_chains);
        chains.sort();
        chains.dedup();
        info!("New chain number: {}", chains.len());
    }


    // write rainbow table header to file
    let header = RainbowTableHeader {
        magic: RAINBOW_TABLE_HEADER_MAGIC,
        num_chain,
        chain_len,
        table_index,
        min_length: table_opts.min_length,
        max_length: table_opts.max_length,
        charset_length: charset.len() as u64
    };
    let header_ptr = unsafe {
        std::slice::from_raw_parts(
            (&header as *const RainbowTableHeader) as *const u8,
            std::mem::size_of::<RainbowTableHeader>()
        )
    };
    output.write_all(&header_ptr).expect("Failed to write rainbow file header");
    output.write_all(&charset).expect("Failed to write charset to header");

    // pad to 8 bytes
    let padding_len = if charset.len() % 8 != 0 {
        8 - charset.len() % 8
    } else { 0 };
    let padding = [0u8; 8];
    output.write_all(&padding[..padding_len]).expect("Failed to write padding to header");

    // write sorted rainbow chains to file
    match output.write(unsafe {
        std::slice::from_raw_parts(
        chains.as_ptr() as *const u8,
        chains.len() * std::mem::size_of::<RainbowChain>(),
    )}
    ) {
        Ok(len) => {
            let total_len = header_ptr.len() + charset.len() + padding_len + len;
            info!("Successfully writing {} bytes to {}", total_len, &output_file);
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
