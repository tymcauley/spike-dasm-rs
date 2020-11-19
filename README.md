# spike-dasm-rs

Rust implementation of [spike's](https://github.com/riscv/riscv-isa-sim)
disassembler.

## How to get RISC-V opcodes

```
git clone https://github.com/riscv/riscv-opcodes.git
cd riscv-opcodes
make inst.rs
mv inst.rs <destination...>
```
