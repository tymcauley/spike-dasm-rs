# spike-dasm-rs

Rust implementation of [spike's](https://github.com/riscv/riscv-isa-sim)
disassembler.

## Setup

Decompress the provided test vectors, which are required for some tests and
benchmarks:

```
cd inputs
tar xf sim-logs.tar.xz
cd ..
```

## Test

Run `cargo test` to run unit tests.

For a larger test over more inputs, run `./test_sim_logs.sh`.

## Benchmark

Run `cargo bench` to run benchmarks.

## Fuzz

### Setup

Install a nightly Rust toolchain (`rustup install nightly`) and [`cargo
fuzz`](https://github.com/rust-fuzz/cargo-fuzz) (`cargo install -f
cargo-fuzz`).

### Run

Run `cargo +nightly fuzz list` to see all available fuzzing targets.

Run `cargo +nightly fuzz run <target>` to run one of the fuzzing targets.

## How to get RISC-V opcodes

```
git clone https://github.com/riscv/riscv-opcodes.git
cd riscv-opcodes
make inst.rs
mv inst.rs <destination...>
```
