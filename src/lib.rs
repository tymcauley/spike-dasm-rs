pub mod csrs;
pub mod disasm;
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

pub enum Extensions {
    I,
    IMC,
    IMAC,
    IMAFDC,
}

impl Extensions {
    pub fn has_m(&self) -> bool {
        match self {
            Extensions::I => false,
            Extensions::IMC => true,
            Extensions::IMAC => true,
            Extensions::IMAFDC => true,
        }
    }

    pub fn has_a(&self) -> bool {
        match self {
            Extensions::I => false,
            Extensions::IMC => false,
            Extensions::IMAC => true,
            Extensions::IMAFDC => true,
        }
    }

    pub fn has_f(&self) -> bool {
        match self {
            Extensions::I => false,
            Extensions::IMC => false,
            Extensions::IMAC => false,
            Extensions::IMAFDC => true,
        }
    }

    pub fn has_d(&self) -> bool {
        match self {
            Extensions::I => false,
            Extensions::IMC => false,
            Extensions::IMAC => false,
            Extensions::IMAFDC => true,
        }
    }

    pub fn has_c(&self) -> bool {
        match self {
            Extensions::I => false,
            Extensions::IMC => true,
            Extensions::IMAC => true,
            Extensions::IMAFDC => true,
        }
    }
}
