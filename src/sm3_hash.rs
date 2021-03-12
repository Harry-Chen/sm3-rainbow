use clap::Clap;
use sm3::my_sm3_impl::my_hash_impl;

#[derive(Clap, Debug)]
#[clap(
    name = "sm3_hash",
    version = "0.1",
    author = "Shengqi Chen <i@harrychen.xyz>",
    about = "Lookup hashes in rainbow tables of SM3 hash algorithm"
)]
pub struct HashOptions {
    plain_text: Vec<String>
}

fn main() {
    env_logger::builder().init();
    let opts: HashOptions = HashOptions::parse();

    if opts.plain_text.is_empty() {
        eprintln!("Input your text to hash:");
        let mut buffer = String::new();
        loop {
            let bytes = std::io::stdin().read_line(&mut buffer).expect("Failed to read stdin");
            if bytes == 0 {
                let hash = my_hash_impl(buffer.trim().as_bytes());
                let hash_hex = hex::encode(hash.as_ref());
                println!("{}", &hash_hex);
                break;
            }
        }
    } else {
        for str in &opts.plain_text {
            let hash = my_hash_impl(str.as_bytes());
            let hash_hex = hex::encode(hash.as_ref());
            println!("{}: {}", &str, &hash_hex);
        }
    }
}
