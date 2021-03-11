use std::fs::{File, OpenOptions};
use std::io::Read;
use std::path::Path;

use clap::Clap;
use indicatif::{ProgressBar, ProgressStyle};
use log::*;
use memmap::{Mmap, MmapOptions};
use rayon::prelude::*;
use sm3::rainbow::{RainbowChain, RainbowIndex, RainbowTableHeader};

mod util;

#[derive(Clap, Debug)]
#[clap(version = "0.1", author = "Shengqi Chen <i@harrychen.xyz>")]
pub struct LookupOptions {
    #[clap(short = 'h', long, required = true)]
    pub hash: Vec<String>,
    #[clap(short = 't', long, required = true)]
    pub table_files: Vec<String>,
}

fn read_rainbow_table(table: &mut File) -> (RainbowTableHeader, Vec<u8>) {
    let mut header: RainbowTableHeader = unsafe { std::mem::zeroed() };

    unsafe {
        let header_ptr = std::slice::from_raw_parts_mut(
            &mut header as *mut _ as *mut u8,
            std::mem::size_of::<RainbowTableHeader>(),
        );
        table
            .read_exact(header_ptr)
            .expect("Cannot read header from table");
    }

    assert!(header.is_valid());

    let mut charset: Vec<u8> = Vec::new();
    charset.resize(header.charset_length as usize, 0);
    table
        .read_exact(charset.as_mut_slice())
        .expect("Cannot read charset from table");

    info!(
        "Table header: {:?}, charset: {}",
        header,
        String::from_utf8_lossy(&charset).to_owned()
    );

    (header, charset)
}


fn run_lookup(opts: &LookupOptions) {

    let mut initialized = false;
    let mut header: RainbowTableHeader = unsafe { std::mem::zeroed() };
    let mut charset: Vec<u8> = Vec::new();
    let mut mapped_tables: Vec<(String, Mmap)> = Vec::new();

    // open all tables and check header
    for f in &opts.table_files {
        info!("Opening rainbow table: {}", &f);
        if !Path::exists(Path::new(&f)) {
            error!("File not exists: {}", &f);
            std::process::exit(1);
        }
        // read header from files
        let mut file = OpenOptions::new()
            .read(true)
            .open(&f)
            .expect(format!("Cannot open table file: {}", &f).as_str());
        let mut read_result = read_rainbow_table(&mut file);
        // check header consistency
        if !initialized {
            header = read_result.0;
            charset.append(&mut read_result.1);
            initialized = true;
        } else if header != read_result.0 || &charset != &read_result.1 {
            error!("Table {} has inconsistent parameters, abort", &f);
            std::process::exit(1);
        }
        // mmap file
        mapped_tables.push((f.to_owned(), unsafe { MmapOptions::new().map(&file).expect("Failed to mmap file") }));
    }

    // calculate offset to rainbow chain data
    let padding_len = if charset.len() % 8 != 0 {
        8 - charset.len() % 8
    } else {
        0
    };
    let data_offset = std::mem::size_of::<RainbowTableHeader>() + padding_len + charset.len();
    info!("Data offset of tables: {}", data_offset);

    // calculate parameters
    let plaintext_len_range = (header.min_length as usize)..(header.max_length + 1) as usize;
    let plaintext_lens = util::generate_cumulative_lengths(&plaintext_len_range, charset.len());
    let chain_len = header.chain_len as usize;
    let num_chain = header.num_chain as usize;
    let plaintext_space_size = plaintext_lens[plaintext_len_range.end - 1];
    info!(
        "Plain text count: {:?}, space size: {}",
        plaintext_lens, plaintext_space_size
    );

    // run on each hash str
    for hash_str in &opts.hash {
        let mut target_hash = [0u8; 32];
        hex::decode_to_slice(hash_str, &mut target_hash).expect("Hash not valid");
        info!("Trying to crack {}\n", &hash_str);

        // show progress bar
        let progress = ProgressBar::new(num_chain as u64);
        progress.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed}/{eta}] [{bar:50.cyan/blue}] {pos}/{len} ({percent}%)",
                )
                .progress_chars("#>-"),
        );

        // store cracked plain text
        let mut all_plain_text: Vec<String> = Vec::new();

        // iterate over each table
        for m in &mapped_tables {

            let filename = &m.0;
            info!("Starting searching in {}\n", &filename);

            // cast data to &[RainbowChain]
            let chain_data = &m.1.as_ref()[data_offset..];
            let chains = unsafe {
                std::slice::from_raw_parts(
                    chain_data.as_ptr() as *const RainbowChain,
                    num_chain,
                )
            };

            progress.reset();

            // find crack
            let mut cracked: Vec<_> = (0..chain_len)
                .into_par_iter()
                .map(|i| {
                    progress.inc(1);
                    // offset on chain
                    let chain_offset = chain_len - 1 - i;
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
                            charset.as_slice(),
                            &plaintext_len_range,
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
                                charset.as_slice(),
                                &plaintext_len_range,
                                plaintext_lens.as_ref(),
                                chain_len as usize,
                                0,
                            ) {
                                Some(result) => {
                                    let plain = String::from_utf8_lossy(&result).into_owned();
                                    info!("Found plain text: {:?}\n", plain);
                                    Some(plain)
                                }
                                None => {
                                    info!("False alarm detected\n");
                                    None
                                },
                            }
                        }
                        Err(_) => {
                            debug!("Target tail not found for step {}\n", i);
                            None
                        }
                    };

                    result
                })
                .filter(|r| !r.is_none())
                .map(|r| r.unwrap())
                .collect();

            info!("Plain text found in table {}: {:?}\n", &filename, &cracked);
            all_plain_text.append(&mut cracked);
        }

        // post precessing
        progress.finish();
        all_plain_text.sort();
        all_plain_text.dedup();

        if all_plain_text.is_empty() {
            error!("Failed to find plain text for {}", &hash_str);
        } else {
            println!("Found plain text for {}: {:?}", &hash_str, &all_plain_text);
        }
    }
}


fn main() {
    env_logger::builder().init();
    let opts: LookupOptions = LookupOptions::parse();
    println!("Program options: {:?}", opts);
    run_lookup(&opts);
}
