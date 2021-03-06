/// Lookup-table for integer register names.
pub const INT_REGISTER_NAMES: [&str; 32] = [
    "x0", "x1", "x2", "x3", "x4", "x5", "x6", "x7", "x8", "x9", "x10", "x11", "x12", "x13", "x14",
    "x15", "x16", "x17", "x18", "x19", "x20", "x21", "x22", "x23", "x24", "x25", "x26", "x27",
    "x28", "x29", "x30", "x31",
];

pub const INT_REGISTER_ABI_NAMES: [&str; 32] = [
    "zero", "ra", "sp", "gp", "tp", "t0", "t1", "t2", "s0", "s1", "a0", "a1", "a2", "a3", "a4",
    "a5", "a6", "a7", "s2", "s3", "s4", "s5", "s6", "s7", "s8", "s9", "s10", "s11", "t3", "t4",
    "t5", "t6",
];

/// Lookup-table for floating point register names.
pub const FP_REGISTER_NAMES: [&str; 32] = [
    "f0", "f1", "f2", "f3", "f4", "f5", "f6", "f7", "f8", "f9", "f10", "f11", "f12", "f13", "f14",
    "f15", "f16", "f17", "f18", "f19", "f20", "f21", "f22", "f23", "f24", "f25", "f26", "f27",
    "f28", "f29", "f30", "f31",
];

pub const FP_REGISTER_ABI_NAMES: [&str; 32] = [
    "ft0", "ft1", "ft2", "ft3", "ft4", "ft5", "ft6", "ft7", "fs0", "fs1", "fa0", "fa1", "fa2",
    "fa3", "fa4", "fa5", "fa6", "fa7", "fs2", "fs3", "fs4", "fs5", "fs6", "fs7", "fs8", "fs9",
    "fs10", "fs11", "ft8", "ft9", "ft10", "ft11",
];

const fn gen_mask(offset: u8, mask_width: u8) -> u32 {
    let mask = (1 << mask_width) - 1;
    mask << offset
}

pub(crate) const MASK_RD: u32 = gen_mask(7, 5);
pub(crate) const MASK_RS1: u32 = gen_mask(15, 5);
pub(crate) const MASK_RS2: u32 = gen_mask(20, 5);
pub(crate) const MASK_I_TYPE_IMM: u32 = gen_mask(20, 12);

pub(crate) const MATCH_RD_EQUALS_RA: u32 = 1 << 7;
pub(crate) const MATCH_RS1_EQUALS_RA: u32 = 1 << 15;
pub(crate) const MATCH_I_TYPE_IMM_EQUALS_NEG1: u32 = gen_mask(20, 12);
pub(crate) const MATCH_I_TYPE_IMM_EQUALS_1: u32 = 1 << 20;
