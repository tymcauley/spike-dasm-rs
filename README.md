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

## How to get RISC-V opcodes

```
git clone https://github.com/riscv/riscv-opcodes.git
cd riscv-opcodes
make inst.rs
mv inst.rs <destination...>
```
