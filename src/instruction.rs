use std::fmt;

use super::csrs;
use super::inst;
use super::registers::{self, INT_REGISTER_ABI_NAMES};
use super::Xlen;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InstructionLen {
    TwoByte,
    FourByte,
}

#[derive(Clone, Copy)]
pub struct InstructionBits {
    pub bits: u32,
    pub length: InstructionLen,
}

impl InstructionBits {
    pub fn new(x: u32) -> Result<Self, String> {
        let inst_len = {
            if x & 0b11 != 0b11 {
                InstructionLen::TwoByte
            } else if x & 0b11100 != 0b11100 {
                InstructionLen::FourByte
            } else {
                // TODO: Maybe a real error type?
                return Err(format!("Unsupported instruction length for {:x}", x));
            }
        };

        let inst_bits = match inst_len {
            InstructionLen::TwoByte => x & 0x0000_ffff,
            InstructionLen::FourByte => x,
        };

        Ok(Self {
            bits: inst_bits,
            length: inst_len,
        })
    }

    fn shift_and_mask(&self, offset: u8, mask_width: u8) -> u32 {
        let mask = (1 << mask_width) - 1;
        (self.bits >> offset) & mask
    }

    fn shift_and_mask_signed(&self, offset: u8, mask_width: u8) -> u32 {
        let signed_bits = self.bits as i32;
        let sign_extend_shift = offset + mask_width;
        assert!(sign_extend_shift <= 32); // Confirm this won't underflow.
        (signed_bits << (32 - sign_extend_shift) >> (32 - mask_width)) as u32
    }

    pub fn get_rd(&self) -> &str {
        let idx = self.shift_and_mask(7, 5);
        INT_REGISTER_ABI_NAMES[idx as usize]
    }

    pub fn get_rs1(&self) -> &str {
        let idx = self.shift_and_mask(15, 5);
        INT_REGISTER_ABI_NAMES[idx as usize]
    }

    pub fn get_rs2(&self) -> &str {
        let idx = self.shift_and_mask(20, 5);
        INT_REGISTER_ABI_NAMES[idx as usize]
    }

    pub fn get_i_imm(&self) -> i32 {
        self.shift_and_mask_signed(20, 12) as i32
    }

    pub fn get_u_imm(&self) -> u32 {
        self.shift_and_mask_signed(12, 20) << 12
    }

    pub fn get_s_imm(&self) -> i32 {
        let lower_imm = self.shift_and_mask(7, 5);
        let upper_imm = self.shift_and_mask_signed(25, 7) << 5;
        (lower_imm + upper_imm) as i32
    }

    pub fn get_big_imm(&self) -> u32 {
        self.get_u_imm() >> 12
    }

    pub fn get_j_imm(&self) -> i32 {
        let imm_10_1 = self.shift_and_mask(21, 10) << 1;
        let imm_11 = self.shift_and_mask(20, 1) << 11;
        let imm_19_12 = self.shift_and_mask(12, 8) << 12;
        let imm_20 = self.shift_and_mask_signed(31, 1) << 20;
        (imm_10_1 + imm_11 + imm_19_12 + imm_20) as i32
    }

    pub fn get_b_imm(&self) -> i32 {
        let imm_4_1 = self.shift_and_mask(8, 4) << 1;
        let imm_10_5 = self.shift_and_mask(25, 6) << 5;
        let imm_11 = self.shift_and_mask(7, 1) << 11;
        let imm_12 = self.shift_and_mask_signed(31, 1) << 12;
        (imm_4_1 + imm_10_5 + imm_11 + imm_12) as i32
    }

    pub fn get_uimm5(&self) -> u32 {
        self.shift_and_mask(15, 5)
    }

    pub fn get_shamt(&self) -> u32 {
        // On RV32 and for the various word-size shifts on RV64 (SLLIW, etc), the shift-amount is a
        // 5-bit number, while it's a 6-bit number for the normal RV64 shift instructions. However,
        // for all of the 5-bit-shift-amount cases, the most-significant-bit is always encoded as a
        // 0. So, we can always use a 6-bit shift-amount.
        self.shift_and_mask(20, 6)
    }

    pub fn get_csr(&self) -> u32 {
        self.shift_and_mask(20, 12)
    }
}

impl fmt::Debug for InstructionBits {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("InstructionBits")
            .field("bits", &format_args!("{:0>8x}", self.bits))
            .field("length", &self.length)
            .finish()
    }
}

// type FmtFn = Box<dyn Fn(&InstructionFilter, InstructionBits) -> String>;
type FmtFn = fn(&InstructionFilter, InstructionBits) -> String;

pub struct InstructionFilter {
    name: &'static str,
    mask: u32,
    r#match: u32,
    pub formatter: FmtFn,
}

impl InstructionFilter {
    pub fn new(name: &'static str, mask: u32, r#match: u32, formatter: FmtFn) -> Self {
        Self {
            name,
            mask,
            r#match,
            formatter,
        }
    }

    pub fn is_eq(&self, other: InstructionBits) -> bool {
        (other.bits & self.mask) == self.r#match
    }
}

impl fmt::Display for InstructionFilter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:<7}", self.name)
    }
}

impl fmt::Debug for InstructionFilter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("InstructionFilter")
            .field("name", &self.name)
            .field("mask", &format_args!("{:0>8x}", self.mask))
            .field("match", &format_args!("{:0>8x}", self.r#match))
            .finish()
    }
}

fn fmt_i_type(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {}, {}",
        inst_filter,
        inst_bits.get_rd(),
        inst_bits.get_rs1(),
        inst_bits.get_i_imm()
    )
}

fn fmt_i_type_shift(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {}, {}",
        inst_filter,
        inst_bits.get_rd(),
        inst_bits.get_rs1(),
        inst_bits.get_shamt()
    )
}

fn fmt_i_type_just_rs1(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!("{} {}", inst_filter, inst_bits.get_rs1(),)
}

fn fmt_i_type_no_rs1(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {}",
        inst_filter,
        inst_bits.get_rd(),
        inst_bits.get_i_imm(),
    )
}

fn fmt_i_type_no_imm(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {}",
        inst_filter,
        inst_bits.get_rd(),
        inst_bits.get_rs1(),
    )
}

fn fmt_u_type(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {:#x}",
        inst_filter,
        inst_bits.get_rd(),
        inst_bits.get_big_imm()
    )
}

fn fmt_r_type(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {}, {}",
        inst_filter,
        inst_bits.get_rd(),
        inst_bits.get_rs1(),
        inst_bits.get_rs2()
    )
}

fn fmt_r_type_no_rs1(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {}",
        inst_filter,
        inst_bits.get_rd(),
        inst_bits.get_rs2()
    )
}

fn fmt_j_type(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    let jump_immediate = inst_bits.get_j_imm();
    let operator = if jump_immediate.is_negative() {
        '-'
    } else {
        '+'
    };
    let jump_immediate_pos = jump_immediate.abs();
    format!(
        "{} {}, pc {} {:#x}",
        inst_filter,
        inst_bits.get_rd(),
        operator,
        jump_immediate_pos
    )
}

fn fmt_j_type_no_rd(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    let jump_immediate = inst_bits.get_j_imm();
    let operator = if jump_immediate.is_negative() {
        '-'
    } else {
        '+'
    };
    let jump_immediate_pos = jump_immediate.abs();
    format!("{} pc {} {:#x}", inst_filter, operator, jump_immediate_pos)
}

fn fmt_b_type(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    let branch_immediate = inst_bits.get_b_imm();
    let operator = if branch_immediate.is_negative() {
        '-'
    } else {
        '+'
    };
    let branch_immediate_pos = branch_immediate.abs();
    format!(
        "{} {}, {}, pc {} {}",
        inst_filter,
        inst_bits.get_rs1(),
        inst_bits.get_rs2(),
        operator,
        branch_immediate_pos
    )
}

fn fmt_b_type_no_rs2(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    let branch_immediate = inst_bits.get_b_imm();
    let operator = if branch_immediate.is_negative() {
        '-'
    } else {
        '+'
    };
    let branch_immediate_pos = branch_immediate.abs();
    format!(
        "{} {}, pc {} {}",
        inst_filter,
        inst_bits.get_rs1(),
        operator,
        branch_immediate_pos
    )
}

fn fmt_load(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {}({})",
        inst_filter,
        inst_bits.get_rd(),
        inst_bits.get_i_imm(),
        inst_bits.get_rs1()
    )
}

fn fmt_store(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {}({})",
        inst_filter,
        inst_bits.get_rs2(),
        inst_bits.get_s_imm(),
        inst_bits.get_rs1()
    )
}

fn fmt_no_args(inst_filter: &InstructionFilter, _inst_bits: InstructionBits) -> String {
    format!("{}", inst_filter)
}

fn fmt_csr(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    let csr_index = inst_bits.get_csr();
    let csr_str = csrs::lookup_csr(csr_index).unwrap_or("unknown");
    format!(
        "{} {}, {}, {}",
        inst_filter,
        inst_bits.get_rd(),
        csr_str,
        inst_bits.get_rs1()
    )
}

fn fmt_csr_no_rs1(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    let csr_index = inst_bits.get_csr();
    let csr_str = csrs::lookup_csr(csr_index).unwrap_or("unknown");
    format!("{} {}, {}", inst_filter, inst_bits.get_rd(), csr_str)
}

fn fmt_csr_no_rd(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    let csr_index = inst_bits.get_csr();
    let csr_str = csrs::lookup_csr(csr_index).unwrap_or("unknown");
    format!("{} {}, {}", inst_filter, csr_str, inst_bits.get_rs1())
}

fn fmt_csr_imm(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    let csr_index = inst_bits.get_csr();
    let csr_str = csrs::lookup_csr(csr_index).unwrap_or("unknown");
    format!(
        "{} {}, {}, {}",
        inst_filter,
        inst_bits.get_rd(),
        csr_str,
        inst_bits.get_uimm5()
    )
}

fn fmt_csr_imm_no_rd(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    let csr_index = inst_bits.get_csr();
    let csr_str = csrs::lookup_csr(csr_index).unwrap_or("unknown");
    format!("{} {}, {}", inst_filter, csr_str, inst_bits.get_uimm5())
}

/// Returns a list of `InstructionFilter` objects to use in the disassembler.
pub fn gen_instructions(_xlen: Xlen) -> Vec<InstructionFilter> {
    // TODO: Fill this out, include extensions.
    // TODO: Pseudo-instructions.
    // TODO: Allow disabling pseudo-instructions.

    vec![
        // Integer-immediate
        //  - pseudo instructions
        InstructionFilter::new(
            "nop",
            inst::MASK_ADDI | registers::MASK_RD | registers::MASK_RS1 | registers::MASK_I_TYPE_IMM,
            inst::MATCH_ADDI,
            fmt_no_args,
        ),
        InstructionFilter::new(
            "li",
            inst::MASK_ADDI | registers::MASK_RS1,
            inst::MATCH_ADDI,
            fmt_i_type_no_rs1,
        ),
        InstructionFilter::new(
            "mv",
            inst::MASK_ADDI | registers::MASK_I_TYPE_IMM,
            inst::MATCH_ADDI,
            fmt_i_type_no_imm,
        ),
        InstructionFilter::new(
            "not",
            inst::MASK_XORI | registers::MASK_I_TYPE_IMM,
            inst::MATCH_XORI | registers::MATCH_I_TYPE_IMM_EQUALS_NEG1,
            fmt_i_type_no_imm,
        ),
        InstructionFilter::new(
            "sext.w",
            inst::MASK_ADDIW | registers::MASK_I_TYPE_IMM,
            inst::MATCH_ADDIW,
            fmt_i_type_no_imm,
        ),
        InstructionFilter::new(
            "seqz",
            inst::MASK_SLTIU | registers::MASK_I_TYPE_IMM,
            inst::MATCH_SLTIU | registers::MATCH_I_TYPE_IMM_EQUALS_1,
            fmt_i_type_no_imm,
        ),
        //  - standard
        InstructionFilter::new("addi", inst::MASK_ADDI, inst::MATCH_ADDI, fmt_i_type),
        InstructionFilter::new("slti", inst::MASK_SLTI, inst::MATCH_SLTI, fmt_i_type),
        InstructionFilter::new("sltiu", inst::MASK_SLTIU, inst::MATCH_SLTIU, fmt_i_type),
        InstructionFilter::new("ori", inst::MASK_ORI, inst::MATCH_ORI, fmt_i_type),
        InstructionFilter::new("xori", inst::MASK_XORI, inst::MATCH_XORI, fmt_i_type),
        InstructionFilter::new("andi", inst::MASK_ANDI, inst::MATCH_ANDI, fmt_i_type),
        InstructionFilter::new("addiw", inst::MASK_ADDIW, inst::MATCH_ADDIW, fmt_i_type),
        // Integer-immediate shift
        // Note that while there are RV32-specific encodings for SLLI, SRLI, and SRAI, the RV64
        // encodings match those instructions as well, since the only difference between those
        // different XLEN settings is the most-significant bit of the 'shamt' field always being 0
        // on RV32.
        InstructionFilter::new("slli", inst::MASK_SLLI, inst::MATCH_SLLI, fmt_i_type_shift),
        InstructionFilter::new("srli", inst::MASK_SRLI, inst::MATCH_SRLI, fmt_i_type_shift),
        InstructionFilter::new("srai", inst::MASK_SRAI, inst::MATCH_SRAI, fmt_i_type_shift),
        InstructionFilter::new(
            "slliw",
            inst::MASK_SLLIW,
            inst::MATCH_SLLIW,
            fmt_i_type_shift,
        ),
        InstructionFilter::new(
            "srliw",
            inst::MASK_SRLIW,
            inst::MATCH_SRLIW,
            fmt_i_type_shift,
        ),
        InstructionFilter::new(
            "sraiw",
            inst::MASK_SRAIW,
            inst::MATCH_SRAIW,
            fmt_i_type_shift,
        ),
        // Upper-immediate
        InstructionFilter::new("lui", inst::MASK_LUI, inst::MATCH_LUI, fmt_u_type),
        InstructionFilter::new("auipc", inst::MASK_AUIPC, inst::MATCH_AUIPC, fmt_u_type),
        // Register-register
        //  - pseudo instructions
        InstructionFilter::new(
            "snez",
            inst::MASK_SLTU | registers::MASK_RS1,
            inst::MATCH_SLTU,
            fmt_r_type_no_rs1,
        ),
        //  - standard
        InstructionFilter::new("add", inst::MASK_ADD, inst::MATCH_ADD, fmt_r_type),
        InstructionFilter::new("slt", inst::MASK_SLT, inst::MATCH_SLT, fmt_r_type),
        InstructionFilter::new("sltu", inst::MASK_SLTU, inst::MATCH_SLTU, fmt_r_type),
        InstructionFilter::new("and", inst::MASK_AND, inst::MATCH_AND, fmt_r_type),
        InstructionFilter::new("or", inst::MASK_OR, inst::MATCH_OR, fmt_r_type),
        InstructionFilter::new("xor", inst::MASK_XOR, inst::MATCH_XOR, fmt_r_type),
        InstructionFilter::new("sll", inst::MASK_SLL, inst::MATCH_SLL, fmt_r_type),
        InstructionFilter::new("srl", inst::MASK_SRL, inst::MATCH_SRL, fmt_r_type),
        InstructionFilter::new("sub", inst::MASK_SUB, inst::MATCH_SUB, fmt_r_type),
        InstructionFilter::new("sra", inst::MASK_SRA, inst::MATCH_SRA, fmt_r_type),
        // Jumps
        //  - pseudo instructions
        InstructionFilter::new(
            "j",
            inst::MASK_JAL | registers::MASK_RD,
            inst::MATCH_JAL,
            fmt_j_type_no_rd,
        ),
        InstructionFilter::new(
            "jal",
            inst::MASK_JAL | registers::MASK_RD,
            inst::MATCH_JAL | registers::MATCH_RD_EQUALS_RA,
            fmt_j_type_no_rd,
        ),
        InstructionFilter::new(
            "ret",
            inst::MASK_JALR | registers::MASK_RD | registers::MASK_RS1 | registers::MASK_I_TYPE_IMM,
            inst::MATCH_JALR | registers::MATCH_RS1_EQUALS_RA,
            fmt_no_args,
        ),
        InstructionFilter::new(
            "jr",
            inst::MASK_JALR | registers::MASK_RD | registers::MASK_I_TYPE_IMM,
            inst::MATCH_JALR,
            fmt_i_type_just_rs1,
        ),
        InstructionFilter::new(
            "jalr",
            inst::MASK_JALR | registers::MASK_RD | registers::MASK_I_TYPE_IMM,
            inst::MATCH_JALR | registers::MATCH_RD_EQUALS_RA,
            fmt_i_type_just_rs1,
        ),
        //  - standard
        InstructionFilter::new("jal", inst::MASK_JAL, inst::MATCH_JAL, fmt_j_type),
        InstructionFilter::new("jalr", inst::MASK_JALR, inst::MATCH_JALR, fmt_i_type),
        // Branches
        //  - pseudo instructions
        InstructionFilter::new(
            "beqz",
            inst::MASK_BEQ | registers::MASK_RS2,
            inst::MATCH_BEQ,
            fmt_b_type_no_rs2,
        ),
        InstructionFilter::new(
            "bnez",
            inst::MASK_BNE | registers::MASK_RS2,
            inst::MATCH_BNE,
            fmt_b_type_no_rs2,
        ),
        InstructionFilter::new(
            "bltz",
            inst::MASK_BLT | registers::MASK_RS2,
            inst::MATCH_BLT,
            fmt_b_type_no_rs2,
        ),
        InstructionFilter::new(
            "bgez",
            inst::MASK_BGE | registers::MASK_RS2,
            inst::MATCH_BGE,
            fmt_b_type_no_rs2,
        ),
        //  - standard
        InstructionFilter::new("beq", inst::MASK_BEQ, inst::MATCH_BEQ, fmt_b_type),
        InstructionFilter::new("bne", inst::MASK_BNE, inst::MATCH_BNE, fmt_b_type),
        InstructionFilter::new("blt", inst::MASK_BLT, inst::MATCH_BLT, fmt_b_type),
        InstructionFilter::new("bltu", inst::MASK_BLTU, inst::MATCH_BLTU, fmt_b_type),
        InstructionFilter::new("bge", inst::MASK_BGE, inst::MATCH_BGE, fmt_b_type),
        InstructionFilter::new("bgeu", inst::MASK_BGEU, inst::MATCH_BGEU, fmt_b_type),
        // Loads
        InstructionFilter::new("lb", inst::MASK_LB, inst::MATCH_LB, fmt_load),
        InstructionFilter::new("lbu", inst::MASK_LBU, inst::MATCH_LBU, fmt_load),
        InstructionFilter::new("lh", inst::MASK_LH, inst::MATCH_LH, fmt_load),
        InstructionFilter::new("lhu", inst::MASK_LHU, inst::MATCH_LHU, fmt_load),
        InstructionFilter::new("lw", inst::MASK_LW, inst::MATCH_LW, fmt_load),
        InstructionFilter::new("lwu", inst::MASK_LWU, inst::MATCH_LWU, fmt_load),
        InstructionFilter::new("ld", inst::MASK_LD, inst::MATCH_LD, fmt_load),
        // Stores
        InstructionFilter::new("sb", inst::MASK_SB, inst::MATCH_SB, fmt_store),
        InstructionFilter::new("sh", inst::MASK_SH, inst::MATCH_SH, fmt_store),
        InstructionFilter::new("sw", inst::MASK_SW, inst::MATCH_SW, fmt_store),
        InstructionFilter::new("sd", inst::MASK_SD, inst::MATCH_SD, fmt_store),
        // Fences
        InstructionFilter::new("fence", inst::MASK_FENCE, inst::MATCH_FENCE, fmt_no_args),
        // Zifencei extension
        InstructionFilter::new(
            "fence.i",
            inst::MASK_FENCE_I,
            inst::MATCH_FENCE_I,
            fmt_no_args,
        ),
        // Environment calls & breakpoints
        InstructionFilter::new("ecall", inst::MASK_ECALL, inst::MATCH_ECALL, fmt_no_args),
        InstructionFilter::new("ebreak", inst::MASK_EBREAK, inst::MATCH_EBREAK, fmt_no_args),
        // Privileged instructions
        InstructionFilter::new("wfi", inst::MASK_WFI, inst::MATCH_WFI, fmt_no_args),
        InstructionFilter::new("mret", inst::MASK_MRET, inst::MATCH_MRET, fmt_no_args),
        InstructionFilter::new("sret", inst::MASK_SRET, inst::MATCH_SRET, fmt_no_args),
        InstructionFilter::new("uret", inst::MASK_URET, inst::MATCH_URET, fmt_no_args),
        // Control and status registers, Zicsr extension
        //  - pseudo instructions
        InstructionFilter::new(
            "csrr",
            inst::MASK_CSRRS | registers::MASK_RS1,
            inst::MATCH_CSRRS,
            fmt_csr_no_rs1,
        ),
        InstructionFilter::new(
            "csrw",
            inst::MASK_CSRRW | registers::MASK_RD,
            inst::MATCH_CSRRW,
            fmt_csr_no_rd,
        ),
        InstructionFilter::new(
            "csrs",
            inst::MASK_CSRRS | registers::MASK_RD,
            inst::MATCH_CSRRS,
            fmt_csr_no_rd,
        ),
        InstructionFilter::new(
            "csrc",
            inst::MASK_CSRRC | registers::MASK_RD,
            inst::MATCH_CSRRC,
            fmt_csr_no_rd,
        ),
        InstructionFilter::new(
            "csrwi",
            inst::MASK_CSRRWI | registers::MASK_RD,
            inst::MATCH_CSRRWI,
            fmt_csr_imm_no_rd,
        ),
        InstructionFilter::new(
            "csrsi",
            inst::MASK_CSRRSI | registers::MASK_RD,
            inst::MATCH_CSRRSI,
            fmt_csr_imm_no_rd,
        ),
        InstructionFilter::new(
            "csrci",
            inst::MASK_CSRRCI | registers::MASK_RD,
            inst::MATCH_CSRRCI,
            fmt_csr_imm_no_rd,
        ),
        //  - standard
        InstructionFilter::new("csrrw", inst::MASK_CSRRW, inst::MATCH_CSRRW, fmt_csr),
        InstructionFilter::new("csrrs", inst::MASK_CSRRS, inst::MATCH_CSRRS, fmt_csr),
        InstructionFilter::new("csrrc", inst::MASK_CSRRC, inst::MATCH_CSRRC, fmt_csr),
        InstructionFilter::new("csrrwi", inst::MASK_CSRRWI, inst::MATCH_CSRRWI, fmt_csr_imm),
        InstructionFilter::new("csrrsi", inst::MASK_CSRRSI, inst::MATCH_CSRRSI, fmt_csr_imm),
        InstructionFilter::new("csrrci", inst::MASK_CSRRCI, inst::MATCH_CSRRCI, fmt_csr_imm),
        // M extension, integer multiplication and division
        InstructionFilter::new("mul", inst::MASK_MUL, inst::MATCH_MUL, fmt_r_type),
        InstructionFilter::new("mulh", inst::MASK_MULH, inst::MATCH_MULH, fmt_r_type),
        InstructionFilter::new("mulhu", inst::MASK_MULHU, inst::MATCH_MULHU, fmt_r_type),
        InstructionFilter::new("mulhsu", inst::MASK_MULHSU, inst::MATCH_MULHSU, fmt_r_type),
        InstructionFilter::new("mulw", inst::MASK_MULW, inst::MATCH_MULW, fmt_r_type),
        InstructionFilter::new("div", inst::MASK_DIV, inst::MATCH_DIV, fmt_r_type),
        InstructionFilter::new("divu", inst::MASK_DIVU, inst::MATCH_DIVU, fmt_r_type),
        InstructionFilter::new("rem", inst::MASK_REM, inst::MATCH_REM, fmt_r_type),
        InstructionFilter::new("remu", inst::MASK_REMU, inst::MATCH_REMU, fmt_r_type),
        InstructionFilter::new("divw", inst::MASK_DIVW, inst::MATCH_DIVW, fmt_r_type),
        InstructionFilter::new("divuw", inst::MASK_DIVUW, inst::MATCH_DIVUW, fmt_r_type),
        InstructionFilter::new("remw", inst::MASK_REMW, inst::MATCH_REMW, fmt_r_type),
        InstructionFilter::new("remuw", inst::MASK_REMUW, inst::MATCH_REMUW, fmt_r_type),
    ]
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn instruction_len() {
        let inst_4byte = InstructionBits::new(0x0000_0003).unwrap();
        let inst_2byte = InstructionBits::new(0x0000_0000).unwrap();

        assert_eq!(inst_4byte.length, InstructionLen::FourByte);
        assert_eq!(inst_2byte.length, InstructionLen::TwoByte);

        assert!(InstructionBits::new(0x0000_001f).is_err());
    }

    #[test]
    fn shift_and_mask() {
        let inst = InstructionBits::new(0x0000_0f03).unwrap();

        assert_eq!(inst.shift_and_mask(8, 3), 0x7);
        assert_eq!(inst.shift_and_mask_signed(8, 3), u32::MAX);
        assert_eq!(inst.shift_and_mask(8, 4), 0xf);
        assert_eq!(inst.shift_and_mask_signed(8, 4), u32::MAX);
        assert_eq!(inst.shift_and_mask(8, 5), 0xf);
        assert_eq!(inst.shift_and_mask_signed(8, 5), 0xf);
    }
}
