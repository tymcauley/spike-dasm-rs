use std::io::{self, BufWriter};

use spike_dasm_rs::disasm::Disassembler;
use spike_dasm_rs::instruction;
use spike_dasm_rs::Xlen;

fn main() {
    let instructions = instruction::gen_instructions(Xlen::Rv64);
    let disasm = Disassembler::new(instructions);

    // Lock stdin and stdout to improve repeated read/write performance.
    let stdin = io::stdin();
    let locked_stdin = stdin.lock();
    let stdout = io::stdout();
    let locked_stdout = stdout.lock();
    let mut buffered_stdout = BufWriter::new(locked_stdout);

    spike_dasm_rs::io::process_streaming_input(locked_stdin, &mut buffered_stdout, disasm).unwrap();
}
