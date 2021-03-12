#!/bin/bash

export RAYON_NUM_THREADS=128

echo 1 table
RUST_LOG=info cargo run --bin generate_rt --release -- -l 5000 -n 8388608 -i 0
RUST_LOG=info RUST_BACKTRACE=1 cargo test --release --bin lookup_rt -- --nocapture

echo 2 tables
RUST_LOG=info cargo run --bin generate_rt --release -- -l 5000 -n 8388608 -i 1
RUST_LOG=info RUST_BACKTRACE=1 cargo test --release --bin lookup_rt -- --nocapture

echo 4 tables
RUST_LOG=info cargo run --bin generate_rt --release -- -l 5000 -n 8388608 -i 2
RUST_LOG=info cargo run --bin generate_rt --release -- -l 5000 -n 8388608 -i 3
RUST_LOG=info RUST_BACKTRACE=1 cargo test --release --bin lookup_rt -- --nocapture

echo 6 tables
RUST_LOG=info cargo run --bin generate_rt --release -- -l 5000 -n 8388608 -i 4
RUST_LOG=info cargo run --bin generate_rt --release -- -l 5000 -n 8388608 -i 5
RUST_LOG=info RUST_BACKTRACE=1 cargo test --release --bin lookup_rt -- --nocapture

echo 8 tables
RUST_LOG=info cargo run --bin generate_rt --release -- -l 5000 -n 8388608 -i 6
RUST_LOG=info cargo run --bin generate_rt --release -- -l 5000 -n 8388608 -i 7
RUST_LOG=info RUST_BACKTRACE=1 cargo test --release --bin lookup_rt -- --nocapture

echo 10 tables
RUST_LOG=info cargo run --bin generate_rt --release -- -l 5000 -n 8388608 -i 8
RUST_LOG=info cargo run --bin generate_rt --release -- -l 5000 -n 8388608 -i 9
RUST_LOG=info RUST_BACKTRACE=1 cargo test --release --bin lookup_rt -- --nocapture

echo 12 tables
RUST_LOG=info cargo run --bin generate_rt --release -- -l 5000 -n 8388608 -i 10
RUST_LOG=info cargo run --bin generate_rt --release -- -l 5000 -n 8388608 -i 11
RUST_LOG=info RUST_BACKTRACE=1 cargo test --release --bin lookup_rt -- --nocapture

echo 14 tables
RUST_LOG=info cargo run --bin generate_rt --release -- -l 5000 -n 8388608 -i 12
RUST_LOG=info cargo run --bin generate_rt --release -- -l 5000 -n 8388608 -i 13
RUST_LOG=info RUST_BACKTRACE=1 cargo test --release --bin lookup_rt -- --nocapture

echo 16 tables
RUST_LOG=info cargo run --bin generate_rt --release -- -l 5000 -n 8388608 -i 14
RUST_LOG=info cargo run --bin generate_rt --release -- -l 5000 -n 8388608 -i 15
RUST_LOG=info RUST_BACKTRACE=1 cargo test --release --bin lookup_rt -- --nocapture

