#![no_main]
use libfuzzer_sys::fuzz_target;

use spike_dasm_rs::disasm::Disassembler;
use spike_dasm_rs::instruction::{self, InstructionBits};
use spike_dasm_rs::parser;
use spike_dasm_rs::{Extensions, Xlen};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        if let Ok((_end, (_begin, x))) = parser::parse_value(s) {
            if let Ok(inst_bits) = InstructionBits::new(x) {
                let instructions =
                    instruction::gen_instructions(Xlen::Rv64, Extensions::IMAFDC, true);
                let disasm = Disassembler::new(instructions);
                let _ = disasm.fmt_inst(inst_bits);
            }
        }
    }
});
