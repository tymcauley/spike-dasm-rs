use std::io::{self, BufWriter};

fn main() {
    // Lock stdin and stdout to improve repeated read/write performance.
    let stdin = io::stdin();
    let locked_stdin = stdin.lock();
    let stdout = io::stdout();
    let locked_stdout = stdout.lock();
    let mut buffered_stdout = BufWriter::new(locked_stdout);

    spike_dasm_rs::io::process_streaming_input(locked_stdin, &mut buffered_stdout).unwrap();
}
