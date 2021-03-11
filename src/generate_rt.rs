use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use clap::Clap;
use indicatif::{ProgressBar, ProgressStyle};
use log::*;
use rand::Rng;
use rayon::prelude::*;
use sm3::rainbow::{RainbowChain, RainbowIndex, RainbowTableHeader, RAINBOW_TABLE_HEADER_MAGIC};

mod util;

#[derive(Clap, Debug)]
#[clap(version = "0.1", author = "Shengqi Chen <i@harrychen.xyz>")]
pub struct GeneratorOptions {
    #[clap(
        short = 'c',
        long,
        default_value = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
    )]
    pub charset: String,
    #[clap(short = 'm', long, default_value = "5")]
    pub min_length: u32,
    #[clap(short = 'M', long, default_value = "6")]
    pub max_length: u32,
    #[clap(short = 'n', long, default_value = "10000")]
    pub num_chain: u64,
    #[clap(short = 'l', long, default_value = "10000")]
    pub chain_len: u64,
    #[clap(short = 'i', long, default_value = "0")]
    pub table_index: u64,
    #[clap(short = 'o', long)]
    pub output_file: Option<String>,
    #[clap(short = 'f', long)]
    pub force_overwrite: bool,
}


fn run_generate(opts: &GeneratorOptions) {
    // read options
    let charset: &[u8] = opts.charset.as_bytes();
    let plaintext_len_range = (opts.min_length as usize)..(opts.max_length + 1) as usize;
    let num_chain = opts.num_chain;
    let chain_len = opts.chain_len;
    let table_index = opts.table_index;
    let plaintext_lens = util::generate_cumulative_lengths(&plaintext_len_range, charset.len());
    let plaintext_space_size = plaintext_lens[plaintext_len_range.end - 1];
    info!(
        "Plain text count: {:?}, space size: {}",
        plaintext_lens, plaintext_space_size
    );

    // try to open file for writing
    let output_file = match &opts.output_file {
        Some(file) => file.to_owned(),
        None => {
            format!(
                "sm3_m{}_M{}_l{}_c{}_i{:04}.dat",
                opts.min_length, opts.max_length, chain_len, num_chain, table_index
            )
        }
    };
    println!("Using {} as output file", output_file);
    if Path::exists(Path::new(&output_file)) {
        warn!("File already exists: {}", &output_file);
        if !opts.force_overwrite {
            std::process::exit(1);
        }
        warn!("Overwriting {} due to force flag", &output_file);
    }
    let mut output = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&output_file)
        .expect("Cannot open output file");


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
    info!(
        "Start generating rainbow chains from index {} to {}",
        start_index, end_index
    );

    let mut chains: Vec<_> = (start_index..end_index)
        .into_par_iter()
        .map(|i| {
            let head = RainbowIndex(i);
            let chain = RainbowChain::from_index(
                head,
                charset,
                &plaintext_len_range,
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
    info!(
        "Table size after removing duplicated tails: {}",
        chains.len()
    );

    // generate from random indices until reaching num_chain
    while chains.len() < num_chain as usize {
        let num_remain_chain = (num_chain as usize) - chains.len();
        info!(
            "Generating remaining {} chains from random numbers",
            num_remain_chain
        );
        // generate random indices
        let mut rng = rand::thread_rng();
        let remaining_index: Vec<_> = (0..num_remain_chain)
            .map(|_| rng.gen_range(0..plaintext_space_size))
            .collect();
        // generate random chains
        let mut random_chains: Vec<_> = (0..num_remain_chain)
            .into_par_iter()
            .map(|i| {
                let head = RainbowIndex(remaining_index[i]);
                let chain = RainbowChain::from_index(
                    head,
                    charset,
                    &plaintext_len_range,
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
        min_length: opts.min_length,
        max_length: opts.max_length,
        charset_length: charset.len() as u64,
    };
    let header_ptr = unsafe {
        std::slice::from_raw_parts(
            (&header as *const RainbowTableHeader) as *const u8,
            std::mem::size_of::<RainbowTableHeader>(),
        )
    };
    output
        .write_all(&header_ptr)
        .expect("Failed to write rainbow file header");
    output
        .write_all(&charset)
        .expect("Failed to write charset to header");

    // pad to 8 bytes
    let padding_len = if charset.len() % 8 != 0 {
        8 - charset.len() % 8
    } else {
        0
    };
    let padding = [0u8; 8];
    output
        .write_all(&padding[..padding_len])
        .expect("Failed to write padding to header");

    // write sorted rainbow chains to file
    match output.write(unsafe {
        std::slice::from_raw_parts(
            chains.as_ptr() as *const u8,
            chains.len() * std::mem::size_of::<RainbowChain>(),
        )
    }) {
        Ok(len) => {
            let total_len = header_ptr.len() + charset.len() + padding_len + len;
            info!(
                "Successfully writing {} bytes to {}",
                total_len, &output_file
            );
        }
        Err(err) => {
            error!("Error writing file: {:?}", err);
            std::process::exit(2);
        }
    }
}


fn main() {
    env_logger::builder().init();
    let opts: GeneratorOptions = GeneratorOptions::parse();
    println!("Program options: {:?}", opts);
    run_generate(&opts);
}
