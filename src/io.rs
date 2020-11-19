use std::io::{self, BufRead, Write};

use super::parser;

pub fn process_streaming_input<I: BufRead, O: Write>(
    istream: I,
    ostream: &mut O,
) -> io::Result<()> {
    for line in istream.lines() {
        let line = line?;
        match parser::parse_value(&line) {
            // TODO: Disassemble `x`
            Ok((end, (begin, x))) => writeln!(ostream, "{}{:#x}{}", begin, x, end)?,
            Err(_) => writeln!(ostream, "{}", line)?,
        }
    }

    // Technically not necessary, as `ostream` will flush when it's dropped, but this will
    // explicitly show any errors when flushing, while flush-upon-drop masks flushing errors.
    ostream.flush()?;

    Ok(())
}
