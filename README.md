# spike-dasm-rs &emsp; [![ci-status]][ci-jobs]

[ci-status]: https://github.com/tymcauley/spike-dasm-rs/workflows/CI/badge.svg
[ci-jobs]: https://github.com/tymcauley/spike-dasm-rs/actions?query=branch%3Amaster

Rust implementation of [spike's](https://github.com/riscv/riscv-isa-sim) RISC-V
disassembler, `spike-dasm`.

## Setup

Decompress the provided test vectors, which are required for some tests and
benchmarks:

```
cd inputs
tar xf sim-logs.tar.xz
cd ..
```

This unpacks two directories:

* `inputs/sim-logs-no-dasm`: raw instruction trace logs from RISC-V
  ISA/benchmark simulations
* `inputs/sim-logs-dasm`: same as `inputs/sim-logs-no-dasm`, but processed by
  `spike-dasm`

You can use any of the files in `inputs/sim-logs-no-dasm` as test cases for
`spike-dasm-rs`, and compare their outputs to the files in
`inputs/sim-logs-dasm`, which shows `spike-dasm`'s behavior.

## Run

`spike-dasm-rs` processes a [Rocket
Chip](https://github.com/chipsalliance/rocket-chip) instruction trace and
disassembles the machine code segments within `DASM(...)` strings.
For example:

```
$ cat inputs/sim-logs-no-dasm/rv64ui-p-simple.out
...
C0:         19 [1] pc=[0000000000010040] W[r10=0000000000010040][1] R[r 0=0000000000000000] R[r 0=0000000000000000] inst=[00000517] DASM(00000517)
C0:         20 [1] pc=[0000000000010044] W[r10=0000000000010000][1] R[r10=0000000000010040] R[r 0=0000000000000000] inst=[fc050513] DASM(fc050513)
C0:         21 [1] pc=[0000000000010048] W[r 0=0000000000000000][1] R[r10=0000000000010000] R[r 0=0000000000000000] inst=[30551073] DASM(30551073)
...
$ cargo run --release < inputs/sim-logs-no-dasm/rv64ui-p-simple.out
...
C0:         19 [1] pc=[0000000000010040] W[r10=0000000000010040][1] R[r 0=0000000000000000] R[r 0=0000000000000000] inst=[00000517] auipc   a0, 0x0
C0:         20 [1] pc=[0000000000010044] W[r10=0000000000010000][1] R[r10=0000000000010040] R[r 0=0000000000000000] inst=[fc050513] addi    a0, a0, -64
C0:         21 [1] pc=[0000000000010048] W[r 0=0000000000000000][1] R[r10=0000000000010000] R[r 0=0000000000000000] inst=[30551073] csrw    mtvec, a0
...
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
