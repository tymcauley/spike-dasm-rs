#![no_main]
use libfuzzer_sys::fuzz_target;

use spike_dasm_rs::instruction::InstructionBits;
use spike_dasm_rs::parser;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        if let Ok((_end, (_begin, x))) = parser::parse_value(s) {
            let _ = InstructionBits::new(x);
        }
    }
});
