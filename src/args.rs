use clap::Clap;

#[derive(Clap, Debug)]
#[clap(version = "0.1", author = "Shengqi Chen <i@harrychen.xyz>")]
pub struct CommonOptions {
    #[clap(short = 'c', long, default_value = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789")]
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
