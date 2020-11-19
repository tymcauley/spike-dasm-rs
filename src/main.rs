use std::io;

fn main() {
    let stdin = io::stdin();
    let locked_stdin = stdin.lock();
    spike_dasm_rs::io::process_streaming_input(locked_stdin).unwrap();
}
