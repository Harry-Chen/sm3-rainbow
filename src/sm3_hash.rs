use clap::Clap;
use sm3::*;

#[derive(Clap, Debug)]
#[clap(
    name = "sm3_hash",
    version = "0.1",
    author = "Shengqi Chen <i@harrychen.xyz>",
    about = "Lookup hashes in rainbow tables of SM3 hash algorithm"
)]
pub struct HashOptions {
    plain_text: Vec<String>,
    #[clap(short = 'i', long, default_value = "my")]
    /// SM3 implementation to use ("my" / "openssl")
    pub implementation: String,
}

fn main() {
    env_logger::builder().init();
    let opts: HashOptions = HashOptions::parse();

    let hasher: Hash = match opts.implementation.to_ascii_lowercase().as_str() {
        "my" => sm3::MY_SM3,
        "openssl" => sm3::OPENSSL_SM3,
        _ => {
            panic!("Unknown implementation: {}", &opts.implementation);
        }
    };

    eprintln!("Using implementation: {}", &opts.implementation);

    if opts.plain_text.is_empty() {
        eprintln!("Input your text to hash:");
        let mut buffer = String::new();
        loop {
            let bytes = std::io::stdin()
                .read_line(&mut buffer)
                .expect("Failed to read stdin");
            if bytes == 0 {
                let hash = hasher(buffer.trim().as_bytes());
                let hash_hex = hex::encode(hash.as_ref());
                println!("{}", &hash_hex);
                break;
            }
        }
    } else {
        for str in &opts.plain_text {
            let hash = hasher(str.as_bytes());
            let hash_hex = hex::encode(hash.as_ref());
            println!("{}: {}", &str, &hash_hex);
        }
    }
}
