use std::io::{self, BufRead, Write};

use super::disasm::Disassembler;
use super::instruction::InstructionBits;
use super::parser;

pub fn process_streaming_input<I: BufRead, O: Write>(
    istream: I,
    ostream: &mut O,
    disasm: Disassembler,
) -> io::Result<()> {
    for line in istream.lines() {
        let line = line?;
        match parser::parse_value(&line) {
            Ok((end, (begin, x))) => {
                let s = match InstructionBits::new(x) {
                    Err(e) => format!("(error interpreting instruction: {})", e),
                    Ok(inst_bits) => disasm
                        .fmt_inst(inst_bits)
                        .unwrap_or_else(|| "unknown".to_string()),
                };
                writeln!(ostream, "{}{}{}", begin, s, end)?;
            }

            // If the parse fails, that means this line doesn't contain anything to disassemble, so
            // fall back to printing the original line.
            Err(_) => writeln!(ostream, "{}", line)?,
        }
    }

    // Technically not necessary, as `ostream` will flush when it's dropped, but this will
    // explicitly show any errors when flushing, while flush-upon-drop masks flushing errors.
    ostream.flush()?;

    Ok(())
}
