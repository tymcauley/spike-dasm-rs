use std::io::{self, BufRead};

use super::parser;

pub fn process_streaming_input<S: BufRead>(stream: S) -> io::Result<()> {
    for line in stream.lines() {
        let line = line?;
        match parser::parse_value(&line) {
            // TODO: Disassemble `x`
            Ok((end, (begin, x))) => println!("{}{:#x}{}", begin, x, end),
            Err(_) => println!("{}", line),
        }
    }

    Ok(())
}
