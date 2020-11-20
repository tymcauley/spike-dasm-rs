#[allow(dead_code)]
pub mod inst;
pub mod instruction;
pub mod io;
pub mod parser;
pub mod registers;

pub enum Xlen {
    Rv32,
    Rv64,
}
