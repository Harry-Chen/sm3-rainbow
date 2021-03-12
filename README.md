# Rainbow table for SM3

[SM3 hashing algorithm](https://en.wikipedia.org/wiki/SM3_(hash_function))
and rainbow table generation & lookup implemented in Rust.
It utilizes [`rayon`](https://github.com/rayon-rs/rayon) for data-parallel acceleration.

## Build on Linux

You must have OpenSSL (>= 1.1.1 and with SM3 enabled) and its development files (headers & libraries) installed to compile this project.
On Debian or Ubuntu, just install `openssl-dev` with `apt`.

Install `rustup`, clone this project, then run `cargo build --release`.

Run binary with `cargo run --release --bin bin_name -- args`.
The default binary to run is `sm3_hash`.
Use `-h` to see the help of any binary.

## Binaries

### `sm3_hash`

Calculate SM3 hash. It can read input from `stdin` or use positional arguments.

Use `-i my` or `-i openssl` to switch implementation.

### `generate_rt`

Generate a rainbow table with the specific parameters:

* charset
* plain text length range
* number of chains in each table
* length of each chain
* table index

See its help for how arguments work and their default values. 

#### Example usage

To generate two rainbow tables (each has 5000 chains and each chain has length 10000)
for all strings in the format of `[a-zA-Z0-9]{5,6}`:

```bash
RUST_LOG=info cargo run --release --bin generate_rt -- -l 10000 -n 5000 -m 5 -M 6 -i 0 # table 0
RUST_LOG=info cargo run --release --bin generate_rt -- -l 10000 -n 5000 -m 5 -M 6 -i 1 # table 1
```

The output file name can be specified by `-o output_file` or automatically synthesized by the parameters above.
The above commands lead to two files: `sm3_m5_M6_l10000_n5000_i000[0-1].dat`

You should generate at lease `key_sapce_size / (chain_len * chain_num)` tables for practical cracking.

Specify `-r` to use random numbers as starting points of rainbow chains instead of sequentially traversing the plain text space.
Specify `-f` to forcibly overwrite output files even if it exists.

#### Environment variables

The following variables can control the behaviour of `generate_rt` and `lookup_rt`.

* `RAYON_NUM_THREADS`: the max concurrency in generation (default to output of `nproc` command)
* `RUST_LOG`: log level (default to `error`, yet `info` is recommended)

### `lookup_rt`

Use generated rainbow tables to lookup hashes. Usage:

```bash
RUST_LOG=info cargo run --release --bin lookup_rt -- -t table*.dat -h hash1 hash2 hash3 ...
```

Note that the argument `table*.dat` needs to be expanded by shell (not `loopup_rt`) to a space-separated list of filenames.

The tables provided to `lookup_rt` must have exactly the same parameters except table index. Otherwise it will abort.

## Tests & Benches

### SM3 algorithm

Run `cargo test --test sm3_tests` to test our implementation of SM3 algorithm.

Run `cargo bench --bench sm3_benches` to run a benchmark on two SM3 implementations (our version v.s. OpenSSL version).

### Rainbow table coverage

Run `cargo test --release --bin lookup_rt -- --nocapture` to test the coverage of all rainbow tables (which must have same parameters) combined in the working directory.

The test will generate 10000 random string according to the parameters of the tables, hash them and try to crack the hashes.
It will report success rate after finishing all tests.

The `test_coverage.sh` contains a script that test the coverage of `1, 2, 4, 6, 8, 10, 12, 14, 16` table incrementally.

## File format

The rainbow table files (`*.dat`) used by `generate_rt` and `lookup_rt` have a simple structure as defined in `RainbowTableHeader`.
It can be described as:

```c++
struct alignas(8) RainbowTableHeader {
    uint64_t magic;
    uint64_t num_chain, chain_len, table_index;
    uint32_t min_length, max_length;
    uint64_t charset_length;
    uint8_t charset[charset_length]; // note: not NUL-terminated
    // zero padding to align to 8 bytes
};
```

Followed by the header are contiguously-stored sorted rainbow chains. There are `num_chain` items in total.
Each chain contains two `uint64_t`, respectively the starting point and tail index of the chain.
