use criterion::{criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use std::fs;
use std::io::{self, BufReader, Cursor};
use std::path::PathBuf;

use spike_dasm_rs::disasm::Disassembler;
use spike_dasm_rs::instruction;
use spike_dasm_rs::{Extensions, Xlen};

const INPUT_DIR: &str = "inputs/sim-logs-no-dasm";

pub fn criterion_benchmark(c: &mut Criterion) {
    // Read input file to string.
    let test_file: PathBuf = [INPUT_DIR, "rv64ui-p-simple.out"].iter().collect();
    let file_data = fs::read_to_string(&test_file).unwrap_or_else(|e| {
        panic!("Failed to read '{}': {}", test_file.display(), e);
    });
    let num_lines = file_data.lines().count();

    let mut group = c.benchmark_group("simple-benchmark");
    group.throughput(Throughput::Elements(num_lines as u64));
    group.bench_function("dasm-simple", |b| {
        // The first closure is benchmark setup, and the second closure is the actual benchmark.
        b.iter_batched(
            || {
                let instructions =
                    instruction::gen_instructions(Xlen::Rv64, Extensions::IMAFDC, true);
                let disasm = Disassembler::new(instructions);

                // Wrap the string so it implements `BufRead`.
                let file_data_cursor = Cursor::new(file_data.as_bytes());
                let file_data_reader = BufReader::new(file_data_cursor);
                (disasm, file_data_reader)
            },
            |(disasm, mut file_data_reader)| {
                spike_dasm_rs::io::process_streaming_input(
                    &mut file_data_reader,
                    &mut io::sink(),
                    disasm,
                )
                .unwrap()
            },
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
