use std::fmt;

use super::csrs;
use super::inst;
use super::registers::{self, FP_REGISTER_ABI_NAMES, INT_REGISTER_ABI_NAMES};
use super::{Extensions, Xlen};

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

    /// Returns the index of register `rd`
    fn get_idx_rd(&self) -> u32 {
        self.shift_and_mask(7, 5)
    }

    /// Returns the index of register `rs1`
    fn get_idx_rs1(&self) -> u32 {
        self.shift_and_mask(15, 5)
    }

    /// Returns the index of register `rs2`
    fn get_idx_rs2(&self) -> u32 {
        self.shift_and_mask(20, 5)
    }

    /// Returns the index of register `rs3`
    fn get_idx_rs3(&self) -> u32 {
        self.shift_and_mask(27, 5)
    }

    /// Returns the index of register `rs1` in compressed instructions
    fn get_idx_c_rs1(&self) -> u32 {
        self.shift_and_mask(7, 5)
    }

    /// Returns the index of register `rs2` in compressed instructions
    fn get_idx_c_rs2(&self) -> u32 {
        self.shift_and_mask(2, 5)
    }

    /// Returns the index of register `rd'` (3-bit register encoding) in compressed instructions
    ///
    /// Note that this is the same index as register `rs2'`.
    fn get_idx_c3_rd(&self) -> u32 {
        self.shift_and_mask(2, 3)
    }

    /// Returns the index of register `rs1'` (3-bit register encoding) in compressed instructions
    fn get_idx_c3_rs1(&self) -> u32 {
        self.shift_and_mask(7, 3)
    }

    /// Returns the ABI name of integer register `rd`
    pub fn get_x_rd(&self) -> &str {
        INT_REGISTER_ABI_NAMES[self.get_idx_rd() as usize]
    }

    /// Returns the ABI name of integer register `rs1`
    pub fn get_x_rs1(&self) -> &str {
        INT_REGISTER_ABI_NAMES[self.get_idx_rs1() as usize]
    }

    /// Returns the ABI name of integer register `rs2`
    pub fn get_x_rs2(&self) -> &str {
        INT_REGISTER_ABI_NAMES[self.get_idx_rs2() as usize]
    }

    /// Returns the ABI name of floating-point register `rd`
    pub fn get_f_rd(&self) -> &str {
        FP_REGISTER_ABI_NAMES[self.get_idx_rd() as usize]
    }

    /// Returns the ABI name of floating-point register `rs1`
    pub fn get_f_rs1(&self) -> &str {
        FP_REGISTER_ABI_NAMES[self.get_idx_rs1() as usize]
    }

    /// Returns the ABI name of floating-point register `rs2`
    pub fn get_f_rs2(&self) -> &str {
        FP_REGISTER_ABI_NAMES[self.get_idx_rs2() as usize]
    }

    /// Returns the ABI name of floating-point register `rs3`
    pub fn get_f_rs3(&self) -> &str {
        FP_REGISTER_ABI_NAMES[self.get_idx_rs3() as usize]
    }

    /// Returns the ABI name of integer register `rs1` in compressed instructions
    pub fn get_x_c_rs1(&self) -> &str {
        INT_REGISTER_ABI_NAMES[self.get_idx_c_rs1() as usize]
    }

    /// Returns the ABI name of integer register `rs2` in compressed instructions
    pub fn get_x_c_rs2(&self) -> &str {
        INT_REGISTER_ABI_NAMES[self.get_idx_c_rs2() as usize]
    }

    /// Returns the ABI name of integer register `rd'` (3-bit register encoding) in compressed
    /// instructions
    ///
    /// Note that this is the same ABI name as integer register `rs2'`.
    pub fn get_x_c3_rd(&self) -> &str {
        INT_REGISTER_ABI_NAMES[(self.get_idx_c3_rd() + 8) as usize]
    }

    /// Returns the ABI name of integer register `rs1'` (3-bit register encoding) in compressed
    /// instructions
    pub fn get_x_c3_rs1(&self) -> &str {
        INT_REGISTER_ABI_NAMES[(self.get_idx_c3_rs1() + 8) as usize]
    }

    /// Returns the ABI name of floating-point register `rd'` (3-bit register encoding) in
    /// compressed instructions
    ///
    /// Note that this is the same ABI name as floating-point register `rs2'`.
    pub fn get_f_c3_rd(&self) -> &str {
        FP_REGISTER_ABI_NAMES[(self.get_idx_c3_rd() + 8) as usize]
    }

    /// Returns the ABI name of floating-point register `rs2` in compressed instructions
    pub fn get_f_c_rs2(&self) -> &str {
        FP_REGISTER_ABI_NAMES[self.get_idx_c_rs2() as usize]
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

    pub fn get_ci_lwsp_imm(&self) -> u32 {
        let imm_4_2 = self.shift_and_mask(4, 3) << 2;
        let imm_5 = self.shift_and_mask(12, 1) << 5;
        let imm_7_6 = self.shift_and_mask(2, 2) << 6;
        imm_4_2 + imm_5 + imm_7_6
    }

    pub fn get_ci_ldsp_imm(&self) -> u32 {
        let imm_4_3 = self.shift_and_mask(5, 2) << 3;
        let imm_5 = self.shift_and_mask(12, 1) << 5;
        let imm_8_6 = self.shift_and_mask(2, 3) << 6;
        imm_4_3 + imm_5 + imm_8_6
    }

    pub fn get_css_swsp_imm(&self) -> u32 {
        let imm_5_2 = self.shift_and_mask(9, 4) << 2;
        let imm_7_6 = self.shift_and_mask(7, 2) << 6;
        imm_5_2 + imm_7_6
    }

    pub fn get_css_sdsp_imm(&self) -> u32 {
        let imm_5_3 = self.shift_and_mask(10, 3) << 3;
        let imm_8_6 = self.shift_and_mask(7, 3) << 6;
        imm_5_3 + imm_8_6
    }

    pub fn get_cl_lw_imm(&self) -> u32 {
        let imm_2 = self.shift_and_mask(6, 1) << 2;
        let imm_5_3 = self.shift_and_mask(10, 3) << 3;
        let imm_6 = self.shift_and_mask(5, 1) << 6;
        imm_2 + imm_5_3 + imm_6
    }

    pub fn get_cl_ld_imm(&self) -> u32 {
        let imm_5_3 = self.shift_and_mask(10, 3) << 3;
        let imm_7_6 = self.shift_and_mask(5, 2) << 6;
        imm_5_3 + imm_7_6
    }

    pub fn get_cj_imm(&self) -> i32 {
        let imm_3_1 = self.shift_and_mask(3, 3) << 1;
        let imm_4 = self.shift_and_mask(11, 1) << 4;
        let imm_5 = self.shift_and_mask(2, 1) << 5;
        let imm_6 = self.shift_and_mask(7, 1) << 6;
        let imm_7 = self.shift_and_mask(6, 1) << 7;
        let imm_9_8 = self.shift_and_mask(9, 2) << 8;
        let imm_10 = self.shift_and_mask(8, 1) << 10;
        let imm_11 = self.shift_and_mask_signed(12, 1) << 11;
        (imm_3_1 + imm_4 + imm_5 + imm_6 + imm_7 + imm_9_8 + imm_10 + imm_11) as i32
    }

    pub fn get_cb_imm(&self) -> i32 {
        let imm_2_1 = self.shift_and_mask(3, 2) << 1;
        let imm_4_3 = self.shift_and_mask(10, 2) << 3;
        let imm_5 = self.shift_and_mask(2, 1) << 5;
        let imm_7_6 = self.shift_and_mask(5, 2) << 6;
        let imm_8 = self.shift_and_mask_signed(12, 1) << 8;
        (imm_2_1 + imm_4_3 + imm_5 + imm_7_6 + imm_8) as i32
    }

    pub fn get_ci_imm(&self) -> i32 {
        let imm_4_0 = self.shift_and_mask(2, 5);
        let imm_5 = self.shift_and_mask_signed(12, 1) << 5;
        (imm_4_0 + imm_5) as i32
    }

    pub fn get_ci_addi16sp_imm(&self) -> i32 {
        let imm_4 = self.shift_and_mask(6, 1) << 4;
        let imm_5 = self.shift_and_mask(2, 1) << 5;
        let imm_6 = self.shift_and_mask(5, 1) << 6;
        let imm_8_7 = self.shift_and_mask(3, 2) << 7;
        let imm_9 = self.shift_and_mask_signed(12, 1) << 9;
        (imm_4 + imm_5 + imm_6 + imm_8_7 + imm_9) as i32
    }

    pub fn get_ciw_addi4spn_imm(&self) -> u32 {
        let imm_2 = self.shift_and_mask(6, 1) << 2;
        let imm_3 = self.shift_and_mask(5, 1) << 3;
        let imm_5_4 = self.shift_and_mask(11, 2) << 4;
        let imm_9_6 = self.shift_and_mask(7, 4) << 6;
        imm_2 + imm_3 + imm_5_4 + imm_9_6
    }

    pub fn get_c_shamt(&self) -> u32 {
        // The shift-amount is the same as the CI immediate, but zero-extended rather than
        // sign-extended.
        (self.get_ci_imm() as u32) & 0x1f
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
        inst_bits.get_x_rd(),
        inst_bits.get_x_rs1(),
        inst_bits.get_i_imm()
    )
}

fn fmt_i_type_shift(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {}, {}",
        inst_filter,
        inst_bits.get_x_rd(),
        inst_bits.get_x_rs1(),
        inst_bits.get_shamt()
    )
}

fn fmt_i_type_just_rs1(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!("{} {}", inst_filter, inst_bits.get_x_rs1())
}

fn fmt_i_type_no_rs1(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {}",
        inst_filter,
        inst_bits.get_x_rd(),
        inst_bits.get_i_imm()
    )
}

fn fmt_i_type_no_imm(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {}",
        inst_filter,
        inst_bits.get_x_rd(),
        inst_bits.get_x_rs1()
    )
}

fn fmt_u_type(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {:#x}",
        inst_filter,
        inst_bits.get_x_rd(),
        inst_bits.get_big_imm()
    )
}

fn fmt_r_type(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {}, {}",
        inst_filter,
        inst_bits.get_x_rd(),
        inst_bits.get_x_rs1(),
        inst_bits.get_x_rs2()
    )
}

fn fmt_r_type_no_rs1(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {}",
        inst_filter,
        inst_bits.get_x_rd(),
        inst_bits.get_x_rs2()
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
        inst_bits.get_x_rd(),
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
        inst_bits.get_x_rs1(),
        inst_bits.get_x_rs2(),
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
        inst_bits.get_x_rs1(),
        operator,
        branch_immediate_pos
    )
}

fn fmt_load(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {}({})",
        inst_filter,
        inst_bits.get_x_rd(),
        inst_bits.get_i_imm(),
        inst_bits.get_x_rs1()
    )
}

fn fmt_store(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {}({})",
        inst_filter,
        inst_bits.get_x_rs2(),
        inst_bits.get_s_imm(),
        inst_bits.get_x_rs1()
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
        inst_bits.get_x_rd(),
        csr_str,
        inst_bits.get_x_rs1()
    )
}

fn fmt_csr_no_rs1(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    let csr_index = inst_bits.get_csr();
    let csr_str = csrs::lookup_csr(csr_index).unwrap_or("unknown");
    format!("{} {}, {}", inst_filter, inst_bits.get_x_rd(), csr_str)
}

fn fmt_csr_no_rd(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    let csr_index = inst_bits.get_csr();
    let csr_str = csrs::lookup_csr(csr_index).unwrap_or("unknown");
    format!("{} {}, {}", inst_filter, csr_str, inst_bits.get_x_rs1())
}

fn fmt_csr_imm(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    let csr_index = inst_bits.get_csr();
    let csr_str = csrs::lookup_csr(csr_index).unwrap_or("unknown");
    format!(
        "{} {}, {}, {}",
        inst_filter,
        inst_bits.get_x_rd(),
        csr_str,
        inst_bits.get_uimm5()
    )
}

fn fmt_csr_imm_no_rd(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    let csr_index = inst_bits.get_csr();
    let csr_str = csrs::lookup_csr(csr_index).unwrap_or("unknown");
    format!("{} {}, {}", inst_filter, csr_str, inst_bits.get_uimm5())
}

fn fmt_amo_lr(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, ({})",
        inst_filter,
        inst_bits.get_x_rd(),
        inst_bits.get_x_rs1()
    )
}

fn fmt_amo(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {}, ({})",
        inst_filter,
        inst_bits.get_x_rd(),
        inst_bits.get_x_rs2(),
        inst_bits.get_x_rs1()
    )
}

fn fmt_fp_load(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {}({})",
        inst_filter,
        inst_bits.get_f_rd(),
        inst_bits.get_i_imm(),
        inst_bits.get_x_rs1()
    )
}

fn fmt_fp_store(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {}({})",
        inst_filter,
        inst_bits.get_f_rs2(),
        inst_bits.get_s_imm(),
        inst_bits.get_x_rs1()
    )
}

fn fmt_fp_r_type(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {}, {}",
        inst_filter,
        inst_bits.get_f_rd(),
        inst_bits.get_f_rs1(),
        inst_bits.get_f_rs2()
    )
}

fn fmt_fp_r_type_no_rs2(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {}",
        inst_filter,
        inst_bits.get_f_rd(),
        inst_bits.get_f_rs1()
    )
}

fn fmt_fp_r_type_with_rs3(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {}, {}, {}",
        inst_filter,
        inst_bits.get_f_rd(),
        inst_bits.get_f_rs1(),
        inst_bits.get_f_rs2(),
        inst_bits.get_f_rs3()
    )
}

fn fmt_fp_r_type_from_int(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {}",
        inst_filter,
        inst_bits.get_f_rd(),
        inst_bits.get_x_rs1()
    )
}

fn fmt_fp_r_type_to_int(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {}",
        inst_filter,
        inst_bits.get_x_rd(),
        inst_bits.get_f_rs1()
    )
}

fn fmt_fp_r_type_int_rd(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {}, {}",
        inst_filter,
        inst_bits.get_x_rd(),
        inst_bits.get_f_rs1(),
        inst_bits.get_f_rs2()
    )
}

fn fmt_ci_type_lwsp(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {}(sp)",
        inst_filter,
        inst_bits.get_x_rd(),
        inst_bits.get_ci_lwsp_imm()
    )
}

fn fmt_ci_type_flwsp(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {}(sp)",
        inst_filter,
        inst_bits.get_f_rd(),
        inst_bits.get_ci_lwsp_imm()
    )
}

fn fmt_ci_type_ldsp(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {}(sp)",
        inst_filter,
        inst_bits.get_x_rd(),
        inst_bits.get_ci_ldsp_imm()
    )
}

fn fmt_ci_type_fldsp(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {}(sp)",
        inst_filter,
        inst_bits.get_f_rd(),
        inst_bits.get_ci_ldsp_imm()
    )
}

fn fmt_css_type_swsp(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {}(sp)",
        inst_filter,
        inst_bits.get_x_c_rs2(),
        inst_bits.get_css_swsp_imm()
    )
}

fn fmt_css_type_fswsp(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {}(sp)",
        inst_filter,
        inst_bits.get_f_c_rs2(),
        inst_bits.get_css_swsp_imm()
    )
}

fn fmt_css_type_sdsp(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {}(sp)",
        inst_filter,
        inst_bits.get_x_c_rs2(),
        inst_bits.get_css_sdsp_imm()
    )
}

fn fmt_css_type_fsdsp(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {}(sp)",
        inst_filter,
        inst_bits.get_f_c_rs2(),
        inst_bits.get_css_sdsp_imm()
    )
}

fn fmt_cl_type_lw(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {}({})",
        inst_filter,
        inst_bits.get_x_c3_rd(),
        inst_bits.get_cl_lw_imm(),
        inst_bits.get_x_c3_rs1()
    )
}

fn fmt_cl_type_ld(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {}({})",
        inst_filter,
        inst_bits.get_x_c3_rd(),
        inst_bits.get_cl_ld_imm(),
        inst_bits.get_x_c3_rs1()
    )
}

fn fmt_cl_type_flw(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {}({})",
        inst_filter,
        inst_bits.get_f_c3_rd(),
        inst_bits.get_cl_lw_imm(),
        inst_bits.get_x_c3_rs1()
    )
}

fn fmt_cl_type_fld(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {}({})",
        inst_filter,
        inst_bits.get_f_c3_rd(),
        inst_bits.get_cl_ld_imm(),
        inst_bits.get_x_c3_rs1()
    )
}

fn fmt_cj_type(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    let jump_immediate = inst_bits.get_cj_imm();
    let operator = if jump_immediate.is_negative() {
        '-'
    } else {
        '+'
    };
    let jump_immediate_pos = jump_immediate.abs();
    format!("{} pc {} {}", inst_filter, operator, jump_immediate_pos)
}

fn fmt_cr_type(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {}",
        inst_filter,
        inst_bits.get_x_c_rs1(),
        inst_bits.get_x_c_rs2()
    )
}

fn fmt_cr_type_no_rs2(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!("{} {}", inst_filter, inst_bits.get_x_c_rs1())
}

fn fmt_cb_type(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    let branch_immediate = inst_bits.get_cb_imm();
    let operator = if branch_immediate.is_negative() {
        '-'
    } else {
        '+'
    };
    let branch_immediate_pos = branch_immediate.abs();
    format!(
        "{} {}, pc {} {}",
        inst_filter,
        inst_bits.get_x_c3_rs1(),
        operator,
        branch_immediate_pos
    )
}

fn fmt_cb_type_shift(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {}",
        inst_filter,
        inst_bits.get_x_c3_rs1(),
        inst_bits.get_c_shamt()
    )
}

fn fmt_cb_type_andi(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {}",
        inst_filter,
        inst_bits.get_x_c3_rs1(),
        inst_bits.get_ci_imm()
    )
}

fn fmt_ci_type(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {}",
        inst_filter,
        inst_bits.get_x_c_rs1(), // rd and rs1 overlap in compressed instructions
        inst_bits.get_ci_imm()
    )
}

fn fmt_ci_type_lui(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    // Mask off the 12 most-significant bits. The `c.lui` instruction takes the 6-bit immediate
    // field `ci_imm`, sign-extends it, and left-shifts it by 12. The disassembly doesn't need to
    // bother showing those 12 trailing zeros, so we just show the 20-bit sign-extended immediate
    // which will be placed in bits 31:12 of `rd`.
    let ci_uimm = (inst_bits.get_ci_imm() as u32) << 12 >> 12;
    format!(
        "{} {}, {:#x}",
        inst_filter,
        inst_bits.get_x_c_rs1(), // rd and rs1 overlap in compressed instructions
        ci_uimm
    )
}

fn fmt_ci_type_addi16sp(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!("{} sp, {}", inst_filter, inst_bits.get_ci_addi16sp_imm())
}

fn fmt_ciw_type_addi4spn(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, sp, {}",
        inst_filter,
        inst_bits.get_x_c3_rd(),
        inst_bits.get_ciw_addi4spn_imm()
    )
}

fn fmt_ci_type_shift(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {}",
        inst_filter,
        inst_bits.get_x_c_rs1(), // rd and rs1 overlap in compressed instructions
        inst_bits.get_c_shamt()
    )
}

fn fmt_ca_type(inst_filter: &InstructionFilter, inst_bits: InstructionBits) -> String {
    format!(
        "{} {}, {}",
        inst_filter,
        inst_bits.get_x_c3_rs1(),
        inst_bits.get_x_c3_rd()
    )
}

/// Returns a list of `InstructionFilter` objects to use in the disassembler.
pub fn gen_instructions(
    xlen: Xlen,
    isa_extensions: Extensions,
    enable_pseudo_instructions: bool,
) -> Vec<InstructionFilter> {
    let mut i_pseudo_instruction_inst_filters = if enable_pseudo_instructions {
        vec![
            // Integer-immediate
            InstructionFilter::new(
                "nop",
                inst::MASK_ADDI
                    | registers::MASK_RD
                    | registers::MASK_RS1
                    | registers::MASK_I_TYPE_IMM,
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
            // Register-register
            InstructionFilter::new(
                "snez",
                inst::MASK_SLTU | registers::MASK_RS1,
                inst::MATCH_SLTU,
                fmt_r_type_no_rs1,
            ),
            // Jumps
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
                inst::MASK_JALR
                    | registers::MASK_RD
                    | registers::MASK_RS1
                    | registers::MASK_I_TYPE_IMM,
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
            // Branches
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
            // Control and status registers, Zicsr extension
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
        ]
    } else {
        vec![]
    };

    let i_inst_filters = {
        let mut xlen_filters = match xlen {
            Xlen::Rv32 => vec![],
            Xlen::Rv64 => vec![
                InstructionFilter::new("addiw", inst::MASK_ADDIW, inst::MATCH_ADDIW, fmt_i_type),
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
                InstructionFilter::new("addw", inst::MASK_ADDW, inst::MATCH_ADDW, fmt_r_type),
                InstructionFilter::new("subw", inst::MASK_SUBW, inst::MATCH_SUBW, fmt_r_type),
                InstructionFilter::new("sllw", inst::MASK_SLLW, inst::MATCH_SLLW, fmt_r_type),
                InstructionFilter::new("srlw", inst::MASK_SRLW, inst::MATCH_SRLW, fmt_r_type),
                InstructionFilter::new("sraw", inst::MASK_SRAW, inst::MATCH_SRAW, fmt_r_type),
                InstructionFilter::new("lwu", inst::MASK_LWU, inst::MATCH_LWU, fmt_load),
                InstructionFilter::new("ld", inst::MASK_LD, inst::MATCH_LD, fmt_load),
                InstructionFilter::new("sd", inst::MASK_SD, inst::MATCH_SD, fmt_store),
            ],
        };

        let global_filters = vec![
            // Integer-immediate
            InstructionFilter::new("addi", inst::MASK_ADDI, inst::MATCH_ADDI, fmt_i_type),
            InstructionFilter::new("slti", inst::MASK_SLTI, inst::MATCH_SLTI, fmt_i_type),
            InstructionFilter::new("sltiu", inst::MASK_SLTIU, inst::MATCH_SLTIU, fmt_i_type),
            InstructionFilter::new("ori", inst::MASK_ORI, inst::MATCH_ORI, fmt_i_type),
            InstructionFilter::new("xori", inst::MASK_XORI, inst::MATCH_XORI, fmt_i_type),
            InstructionFilter::new("andi", inst::MASK_ANDI, inst::MATCH_ANDI, fmt_i_type),
            // Integer-immediate shift
            // Note that while there are RV32-specific encodings for SLLI, SRLI, and SRAI, the RV64
            // encodings match those instructions as well, since the only difference between those
            // different XLEN settings is the most-significant bit of the 'shamt' field always
            // being 0 on RV32.
            InstructionFilter::new("slli", inst::MASK_SLLI, inst::MATCH_SLLI, fmt_i_type_shift),
            InstructionFilter::new("srli", inst::MASK_SRLI, inst::MATCH_SRLI, fmt_i_type_shift),
            InstructionFilter::new("srai", inst::MASK_SRAI, inst::MATCH_SRAI, fmt_i_type_shift),
            // Upper-immediate
            InstructionFilter::new("lui", inst::MASK_LUI, inst::MATCH_LUI, fmt_u_type),
            InstructionFilter::new("auipc", inst::MASK_AUIPC, inst::MATCH_AUIPC, fmt_u_type),
            // Register-register
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
            InstructionFilter::new("jal", inst::MASK_JAL, inst::MATCH_JAL, fmt_j_type),
            InstructionFilter::new("jalr", inst::MASK_JALR, inst::MATCH_JALR, fmt_i_type),
            // Branches
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
            // Stores
            InstructionFilter::new("sb", inst::MASK_SB, inst::MATCH_SB, fmt_store),
            InstructionFilter::new("sh", inst::MASK_SH, inst::MATCH_SH, fmt_store),
            InstructionFilter::new("sw", inst::MASK_SW, inst::MATCH_SW, fmt_store),
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
            InstructionFilter::new("csrrw", inst::MASK_CSRRW, inst::MATCH_CSRRW, fmt_csr),
            InstructionFilter::new("csrrs", inst::MASK_CSRRS, inst::MATCH_CSRRS, fmt_csr),
            InstructionFilter::new("csrrc", inst::MASK_CSRRC, inst::MATCH_CSRRC, fmt_csr),
            InstructionFilter::new("csrrwi", inst::MASK_CSRRWI, inst::MATCH_CSRRWI, fmt_csr_imm),
            InstructionFilter::new("csrrsi", inst::MASK_CSRRSI, inst::MATCH_CSRRSI, fmt_csr_imm),
            InstructionFilter::new("csrrci", inst::MASK_CSRRCI, inst::MATCH_CSRRCI, fmt_csr_imm),
        ];

        xlen_filters.extend(global_filters);
        xlen_filters
    };

    // M extension, integer multiplication and division
    let m_inst_filters = if isa_extensions.has_m() {
        vec![
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
    } else {
        vec![]
    };

    // A extension, atomic instructions
    let a_inst_filters = if isa_extensions.has_a() {
        vec![
            InstructionFilter::new("lr.w", inst::MASK_LR_W, inst::MATCH_LR_W, fmt_amo_lr),
            InstructionFilter::new("sc.w", inst::MASK_SC_W, inst::MATCH_SC_W, fmt_amo),
            InstructionFilter::new("lr.d", inst::MASK_LR_D, inst::MATCH_LR_D, fmt_amo_lr),
            InstructionFilter::new("sc.d", inst::MASK_SC_D, inst::MATCH_SC_D, fmt_amo),
            InstructionFilter::new(
                "amoswap.w",
                inst::MASK_AMOSWAP_W,
                inst::MATCH_AMOSWAP_W,
                fmt_amo,
            ),
            InstructionFilter::new(
                "amoswap.d",
                inst::MASK_AMOSWAP_D,
                inst::MATCH_AMOSWAP_D,
                fmt_amo,
            ),
            InstructionFilter::new(
                "amoadd.w",
                inst::MASK_AMOADD_W,
                inst::MATCH_AMOADD_W,
                fmt_amo,
            ),
            InstructionFilter::new(
                "amoadd.d",
                inst::MASK_AMOADD_D,
                inst::MATCH_AMOADD_D,
                fmt_amo,
            ),
            InstructionFilter::new(
                "amoand.w",
                inst::MASK_AMOAND_W,
                inst::MATCH_AMOAND_W,
                fmt_amo,
            ),
            InstructionFilter::new(
                "amoand.d",
                inst::MASK_AMOAND_D,
                inst::MATCH_AMOAND_D,
                fmt_amo,
            ),
            InstructionFilter::new("amoor.w", inst::MASK_AMOOR_W, inst::MATCH_AMOOR_W, fmt_amo),
            InstructionFilter::new("amoor.d", inst::MASK_AMOOR_D, inst::MATCH_AMOOR_D, fmt_amo),
            InstructionFilter::new(
                "amoxor.w",
                inst::MASK_AMOXOR_W,
                inst::MATCH_AMOXOR_W,
                fmt_amo,
            ),
            InstructionFilter::new(
                "amoxor.d",
                inst::MASK_AMOXOR_D,
                inst::MATCH_AMOXOR_D,
                fmt_amo,
            ),
            InstructionFilter::new(
                "amomax.w",
                inst::MASK_AMOMAX_W,
                inst::MATCH_AMOMAX_W,
                fmt_amo,
            ),
            InstructionFilter::new(
                "amomax.d",
                inst::MASK_AMOMAX_D,
                inst::MATCH_AMOMAX_D,
                fmt_amo,
            ),
            InstructionFilter::new(
                "amomaxu.w",
                inst::MASK_AMOMAXU_W,
                inst::MATCH_AMOMAXU_W,
                fmt_amo,
            ),
            InstructionFilter::new(
                "amomaxu.d",
                inst::MASK_AMOMAXU_D,
                inst::MATCH_AMOMAXU_D,
                fmt_amo,
            ),
            InstructionFilter::new(
                "amomin.w",
                inst::MASK_AMOMIN_W,
                inst::MATCH_AMOMIN_W,
                fmt_amo,
            ),
            InstructionFilter::new(
                "amomin.d",
                inst::MASK_AMOMIN_D,
                inst::MATCH_AMOMIN_D,
                fmt_amo,
            ),
            InstructionFilter::new(
                "amominu.w",
                inst::MASK_AMOMINU_W,
                inst::MATCH_AMOMINU_W,
                fmt_amo,
            ),
            InstructionFilter::new(
                "amominu.d",
                inst::MASK_AMOMINU_D,
                inst::MATCH_AMOMINU_D,
                fmt_amo,
            ),
        ]
    } else {
        vec![]
    };

    // F extension, single-precision floating-point
    let f_inst_filters = if isa_extensions.has_f() {
        vec![
            InstructionFilter::new("flw", inst::MASK_FLW, inst::MATCH_FLW, fmt_fp_load),
            InstructionFilter::new("fsw", inst::MASK_FSW, inst::MATCH_FSW, fmt_fp_store),
            InstructionFilter::new(
                "fadd.s",
                inst::MASK_FADD_S,
                inst::MATCH_FADD_S,
                fmt_fp_r_type,
            ),
            InstructionFilter::new(
                "fsub.s",
                inst::MASK_FSUB_S,
                inst::MATCH_FSUB_S,
                fmt_fp_r_type,
            ),
            InstructionFilter::new(
                "fmul.s",
                inst::MASK_FMUL_S,
                inst::MATCH_FMUL_S,
                fmt_fp_r_type,
            ),
            InstructionFilter::new(
                "fdiv.s",
                inst::MASK_FDIV_S,
                inst::MATCH_FDIV_S,
                fmt_fp_r_type,
            ),
            InstructionFilter::new(
                "fsqrt.s",
                inst::MASK_FSQRT_S,
                inst::MATCH_FSQRT_S,
                fmt_fp_r_type_no_rs2,
            ),
            InstructionFilter::new(
                "fmin.s",
                inst::MASK_FMIN_S,
                inst::MATCH_FMIN_S,
                fmt_fp_r_type,
            ),
            InstructionFilter::new(
                "fmax.s",
                inst::MASK_FMAX_S,
                inst::MATCH_FMAX_S,
                fmt_fp_r_type,
            ),
            InstructionFilter::new(
                "fmadd.s",
                inst::MASK_FMADD_S,
                inst::MATCH_FMADD_S,
                fmt_fp_r_type_with_rs3,
            ),
            InstructionFilter::new(
                "fnmadd.s",
                inst::MASK_FNMADD_S,
                inst::MATCH_FNMADD_S,
                fmt_fp_r_type_with_rs3,
            ),
            InstructionFilter::new(
                "fmsub.s",
                inst::MASK_FMSUB_S,
                inst::MATCH_FMSUB_S,
                fmt_fp_r_type_with_rs3,
            ),
            InstructionFilter::new(
                "fnmsub.s",
                inst::MASK_FNMSUB_S,
                inst::MATCH_FNMSUB_S,
                fmt_fp_r_type_with_rs3,
            ),
            InstructionFilter::new(
                "fcvt.s.w",
                inst::MASK_FCVT_S_W,
                inst::MATCH_FCVT_S_W,
                fmt_fp_r_type_from_int,
            ),
            InstructionFilter::new(
                "fcvt.s.l",
                inst::MASK_FCVT_S_L,
                inst::MATCH_FCVT_S_L,
                fmt_fp_r_type_from_int,
            ),
            InstructionFilter::new(
                "fcvt.s.wu",
                inst::MASK_FCVT_S_WU,
                inst::MATCH_FCVT_S_WU,
                fmt_fp_r_type_from_int,
            ),
            InstructionFilter::new(
                "fcvt.s.lu",
                inst::MASK_FCVT_S_LU,
                inst::MATCH_FCVT_S_LU,
                fmt_fp_r_type_from_int,
            ),
            InstructionFilter::new(
                "fcvt.w.s",
                inst::MASK_FCVT_W_S,
                inst::MATCH_FCVT_W_S,
                fmt_fp_r_type_to_int,
            ),
            InstructionFilter::new(
                "fcvt.l.s",
                inst::MASK_FCVT_L_S,
                inst::MATCH_FCVT_L_S,
                fmt_fp_r_type_to_int,
            ),
            InstructionFilter::new(
                "fcvt.wu.s",
                inst::MASK_FCVT_WU_S,
                inst::MATCH_FCVT_WU_S,
                fmt_fp_r_type_to_int,
            ),
            InstructionFilter::new(
                "fcvt.lu.s",
                inst::MASK_FCVT_LU_S,
                inst::MATCH_FCVT_LU_S,
                fmt_fp_r_type_to_int,
            ),
            InstructionFilter::new(
                "fsgnj.s",
                inst::MASK_FSGNJ_S,
                inst::MATCH_FSGNJ_S,
                fmt_fp_r_type,
            ),
            InstructionFilter::new(
                "fsgnjn.s",
                inst::MASK_FSGNJN_S,
                inst::MATCH_FSGNJN_S,
                fmt_fp_r_type,
            ),
            InstructionFilter::new(
                "fsgnjx.s",
                inst::MASK_FSGNJX_S,
                inst::MATCH_FSGNJX_S,
                fmt_fp_r_type,
            ),
            InstructionFilter::new(
                "fmv.w.x",
                inst::MASK_FMV_W_X,
                inst::MATCH_FMV_W_X,
                fmt_fp_r_type_from_int,
            ),
            InstructionFilter::new(
                "fmv.x.w",
                inst::MASK_FMV_X_W,
                inst::MATCH_FMV_X_W,
                fmt_fp_r_type_to_int,
            ),
            InstructionFilter::new(
                "feq.s",
                inst::MASK_FEQ_S,
                inst::MATCH_FEQ_S,
                fmt_fp_r_type_int_rd,
            ),
            InstructionFilter::new(
                "flt.s",
                inst::MASK_FLT_S,
                inst::MATCH_FLT_S,
                fmt_fp_r_type_int_rd,
            ),
            InstructionFilter::new(
                "fle.s",
                inst::MASK_FLE_S,
                inst::MATCH_FLE_S,
                fmt_fp_r_type_int_rd,
            ),
            InstructionFilter::new(
                "fclass.s",
                inst::MASK_FCLASS_S,
                inst::MATCH_FCLASS_S,
                fmt_fp_r_type_to_int,
            ),
        ]
    } else {
        vec![]
    };

    // D extension, double-precision floating-point
    let d_inst_filters = if isa_extensions.has_d() {
        vec![
            InstructionFilter::new("fld", inst::MASK_FLD, inst::MATCH_FLD, fmt_fp_load),
            InstructionFilter::new("fsd", inst::MASK_FSD, inst::MATCH_FSD, fmt_fp_store),
            InstructionFilter::new(
                "fadd.d",
                inst::MASK_FADD_D,
                inst::MATCH_FADD_D,
                fmt_fp_r_type,
            ),
            InstructionFilter::new(
                "fsub.d",
                inst::MASK_FSUB_D,
                inst::MATCH_FSUB_D,
                fmt_fp_r_type,
            ),
            InstructionFilter::new(
                "fmul.d",
                inst::MASK_FMUL_D,
                inst::MATCH_FMUL_D,
                fmt_fp_r_type,
            ),
            InstructionFilter::new(
                "fdiv.d",
                inst::MASK_FDIV_D,
                inst::MATCH_FDIV_D,
                fmt_fp_r_type,
            ),
            InstructionFilter::new(
                "fsqrt.d",
                inst::MASK_FSQRT_D,
                inst::MATCH_FSQRT_D,
                fmt_fp_r_type_no_rs2,
            ),
            InstructionFilter::new(
                "fmin.d",
                inst::MASK_FMIN_D,
                inst::MATCH_FMIN_D,
                fmt_fp_r_type,
            ),
            InstructionFilter::new(
                "fmax.d",
                inst::MASK_FMAX_D,
                inst::MATCH_FMAX_D,
                fmt_fp_r_type,
            ),
            InstructionFilter::new(
                "fmadd.d",
                inst::MASK_FMADD_D,
                inst::MATCH_FMADD_D,
                fmt_fp_r_type_with_rs3,
            ),
            InstructionFilter::new(
                "fnmadd.d",
                inst::MASK_FNMADD_D,
                inst::MATCH_FNMADD_D,
                fmt_fp_r_type_with_rs3,
            ),
            InstructionFilter::new(
                "fmsub.d",
                inst::MASK_FMSUB_D,
                inst::MATCH_FMSUB_D,
                fmt_fp_r_type_with_rs3,
            ),
            InstructionFilter::new(
                "fnmsub.d",
                inst::MASK_FNMSUB_D,
                inst::MATCH_FNMSUB_D,
                fmt_fp_r_type_with_rs3,
            ),
            InstructionFilter::new(
                "fcvt.d.w",
                inst::MASK_FCVT_D_W,
                inst::MATCH_FCVT_D_W,
                fmt_fp_r_type_from_int,
            ),
            InstructionFilter::new(
                "fcvt.d.l",
                inst::MASK_FCVT_D_L,
                inst::MATCH_FCVT_D_L,
                fmt_fp_r_type_from_int,
            ),
            InstructionFilter::new(
                "fcvt.d.wu",
                inst::MASK_FCVT_D_WU,
                inst::MATCH_FCVT_D_WU,
                fmt_fp_r_type_from_int,
            ),
            InstructionFilter::new(
                "fcvt.d.lu",
                inst::MASK_FCVT_D_LU,
                inst::MATCH_FCVT_D_LU,
                fmt_fp_r_type_from_int,
            ),
            InstructionFilter::new(
                "fcvt.w.d",
                inst::MASK_FCVT_W_D,
                inst::MATCH_FCVT_W_D,
                fmt_fp_r_type_to_int,
            ),
            InstructionFilter::new(
                "fcvt.l.d",
                inst::MASK_FCVT_L_D,
                inst::MATCH_FCVT_L_D,
                fmt_fp_r_type_to_int,
            ),
            InstructionFilter::new(
                "fcvt.wu.d",
                inst::MASK_FCVT_WU_D,
                inst::MATCH_FCVT_WU_D,
                fmt_fp_r_type_to_int,
            ),
            InstructionFilter::new(
                "fcvt.lu.d",
                inst::MASK_FCVT_LU_D,
                inst::MATCH_FCVT_LU_D,
                fmt_fp_r_type_to_int,
            ),
            InstructionFilter::new(
                "fcvt.s.d",
                inst::MASK_FCVT_S_D,
                inst::MATCH_FCVT_S_D,
                fmt_fp_r_type_no_rs2,
            ),
            InstructionFilter::new(
                "fcvt.d.s",
                inst::MASK_FCVT_D_S,
                inst::MATCH_FCVT_D_S,
                fmt_fp_r_type_no_rs2,
            ),
            InstructionFilter::new(
                "fsgnj.d",
                inst::MASK_FSGNJ_D,
                inst::MATCH_FSGNJ_D,
                fmt_fp_r_type,
            ),
            InstructionFilter::new(
                "fsgnjn.d",
                inst::MASK_FSGNJN_D,
                inst::MATCH_FSGNJN_D,
                fmt_fp_r_type,
            ),
            InstructionFilter::new(
                "fsgnjx.d",
                inst::MASK_FSGNJX_D,
                inst::MATCH_FSGNJX_D,
                fmt_fp_r_type,
            ),
            InstructionFilter::new(
                "fmv.x.d",
                inst::MASK_FMV_X_D,
                inst::MATCH_FMV_X_D,
                fmt_fp_r_type_to_int,
            ),
            InstructionFilter::new(
                "fmv.d.x",
                inst::MASK_FMV_D_X,
                inst::MATCH_FMV_D_X,
                fmt_fp_r_type_from_int,
            ),
            InstructionFilter::new(
                "feq.d",
                inst::MASK_FEQ_D,
                inst::MATCH_FEQ_D,
                fmt_fp_r_type_int_rd,
            ),
            InstructionFilter::new(
                "flt.d",
                inst::MASK_FLT_D,
                inst::MATCH_FLT_D,
                fmt_fp_r_type_int_rd,
            ),
            InstructionFilter::new(
                "fle.d",
                inst::MASK_FLE_D,
                inst::MATCH_FLE_D,
                fmt_fp_r_type_int_rd,
            ),
            InstructionFilter::new(
                "fclass.d",
                inst::MASK_FCLASS_D,
                inst::MATCH_FCLASS_D,
                fmt_fp_r_type_to_int,
            ),
        ]
    } else {
        vec![]
    };

    // C extension, compressed instructions
    let c_inst_filters = if isa_extensions.has_c() {
        let mut pseudo_instruction_filters = if enable_pseudo_instructions {
            vec![InstructionFilter::new(
                "ret",
                inst::MASK_C_JR | registers::MASK_RD,
                inst::MATCH_C_JR | registers::MATCH_RD_EQUALS_RA,
                fmt_no_args,
            )]
        } else {
            vec![]
        };

        let xlen_filters = match xlen {
            Xlen::Rv32 => {
                let mut rv32_filters = vec![InstructionFilter::new(
                    "c.jal",
                    inst::MASK_C_JAL,
                    inst::MATCH_C_JAL,
                    fmt_cj_type,
                )];
                let rv32f_filters = if isa_extensions.has_f() {
                    vec![
                        InstructionFilter::new(
                            "c.flwsp",
                            inst::MASK_C_FLWSP,
                            inst::MATCH_C_FLWSP,
                            fmt_ci_type_flwsp,
                        ),
                        InstructionFilter::new(
                            "c.fswsp",
                            inst::MASK_C_FSWSP,
                            inst::MATCH_C_FSWSP,
                            fmt_css_type_fswsp,
                        ),
                        InstructionFilter::new(
                            "c.flw",
                            inst::MASK_C_FLW,
                            inst::MATCH_C_FLW,
                            fmt_cl_type_flw,
                        ),
                        InstructionFilter::new(
                            "c.fsw",
                            inst::MASK_C_FSW,
                            inst::MATCH_C_FSW,
                            fmt_cl_type_flw,
                        ),
                    ]
                } else {
                    vec![]
                };
                rv32_filters.extend(rv32f_filters);
                rv32_filters
            }

            Xlen::Rv64 => vec![
                InstructionFilter::new(
                    "c.ldsp",
                    inst::MASK_C_LDSP,
                    inst::MATCH_C_LDSP,
                    fmt_ci_type_ldsp,
                ),
                InstructionFilter::new(
                    "c.sdsp",
                    inst::MASK_C_SDSP,
                    inst::MATCH_C_SDSP,
                    fmt_css_type_sdsp,
                ),
                InstructionFilter::new("c.ld", inst::MASK_C_LD, inst::MATCH_C_LD, fmt_cl_type_ld),
                InstructionFilter::new("c.sd", inst::MASK_C_SD, inst::MATCH_C_SD, fmt_cl_type_ld),
                InstructionFilter::new(
                    "c.addiw",
                    inst::MASK_C_ADDIW,
                    inst::MATCH_C_ADDIW,
                    fmt_ci_type,
                ),
                InstructionFilter::new(
                    "c.addw",
                    inst::MASK_C_ADDW,
                    inst::MATCH_C_ADDW,
                    fmt_ca_type,
                ),
                InstructionFilter::new(
                    "c.subw",
                    inst::MASK_C_SUBW,
                    inst::MATCH_C_SUBW,
                    fmt_ca_type,
                ),
            ],
        };

        let d_filters = if isa_extensions.has_d() {
            vec![
                InstructionFilter::new(
                    "c.fldsp",
                    inst::MASK_C_FLDSP,
                    inst::MATCH_C_FLDSP,
                    fmt_ci_type_fldsp,
                ),
                InstructionFilter::new(
                    "c.fsdsp",
                    inst::MASK_C_FSDSP,
                    inst::MATCH_C_FSDSP,
                    fmt_css_type_fsdsp,
                ),
                InstructionFilter::new(
                    "c.fld",
                    inst::MASK_C_FLD,
                    inst::MATCH_C_FLD,
                    fmt_cl_type_fld,
                ),
                InstructionFilter::new(
                    "c.fsd",
                    inst::MASK_C_FSD,
                    inst::MATCH_C_FSD,
                    fmt_cl_type_fld,
                ),
            ]
        } else {
            vec![]
        };

        let global_filters = vec![
            InstructionFilter::new(
                "c.lwsp",
                inst::MASK_C_LWSP,
                inst::MATCH_C_LWSP,
                fmt_ci_type_lwsp,
            ),
            InstructionFilter::new(
                "c.swsp",
                inst::MASK_C_SWSP,
                inst::MATCH_C_SWSP,
                fmt_css_type_swsp,
            ),
            InstructionFilter::new("c.lw", inst::MASK_C_LW, inst::MATCH_C_LW, fmt_cl_type_lw),
            InstructionFilter::new("c.sw", inst::MASK_C_SW, inst::MATCH_C_SW, fmt_cl_type_lw),
            InstructionFilter::new(
                "c.ebreak",
                inst::MASK_C_EBREAK,
                inst::MATCH_C_EBREAK,
                fmt_no_args,
            ),
            InstructionFilter::new("c.j", inst::MASK_C_J, inst::MATCH_C_J, fmt_cj_type),
            InstructionFilter::new(
                "c.jr",
                inst::MASK_C_JR,
                inst::MATCH_C_JR,
                fmt_cr_type_no_rs2,
            ),
            // `c.jalr` must come after `c.ebreak`, otherwise the `c.jalr` filter will match any
            // `c.ebreak` instructions.
            InstructionFilter::new(
                "c.jalr",
                inst::MASK_C_JALR,
                inst::MATCH_C_JALR,
                fmt_cr_type_no_rs2,
            ),
            InstructionFilter::new("c.beqz", inst::MASK_C_BEQZ, inst::MATCH_C_BEQZ, fmt_cb_type),
            InstructionFilter::new("c.bnez", inst::MASK_C_BNEZ, inst::MATCH_C_BNEZ, fmt_cb_type),
            InstructionFilter::new("c.li", inst::MASK_C_LI, inst::MATCH_C_LI, fmt_ci_type),
            InstructionFilter::new("c.nop", inst::MASK_C_NOP, inst::MATCH_C_NOP, fmt_no_args),
            // `c.addi` must come after `c.nop`, otherwise the `c.addi` filter will match any
            // `c.nop` instructions.
            InstructionFilter::new("c.addi", inst::MASK_C_ADDI, inst::MATCH_C_ADDI, fmt_ci_type),
            InstructionFilter::new(
                "c.addi16sp",
                inst::MASK_C_ADDI16SP,
                inst::MATCH_C_ADDI16SP,
                fmt_ci_type_addi16sp,
            ),
            InstructionFilter::new(
                "c.addi4spn",
                inst::MASK_C_ADDI4SPN,
                inst::MATCH_C_ADDI4SPN,
                fmt_ciw_type_addi4spn,
            ),
            InstructionFilter::new(
                "c.slli",
                inst::MASK_C_SLLI,
                inst::MATCH_C_SLLI,
                fmt_ci_type_shift,
            ),
            InstructionFilter::new(
                "c.srli",
                inst::MASK_C_SRLI,
                inst::MATCH_C_SRLI,
                fmt_cb_type_shift,
            ),
            InstructionFilter::new(
                "c.srai",
                inst::MASK_C_SRAI,
                inst::MATCH_C_SRAI,
                fmt_cb_type_shift,
            ),
            InstructionFilter::new(
                "c.andi",
                inst::MASK_C_ANDI,
                inst::MATCH_C_ANDI,
                fmt_cb_type_andi,
            ),
            // `c.mv` must come after `c.jr`, otherwise the `c.mv` filter will match any `c.jr`
            // instructions.
            InstructionFilter::new("c.mv", inst::MASK_C_MV, inst::MATCH_C_MV, fmt_cr_type),
            // `c.add` must come after `c.jalr` and `c.ebreak`, otherwise the `c.add` filter will
            // match any `c.jalr`/`c.ebreak` instructions.
            InstructionFilter::new("c.add", inst::MASK_C_ADD, inst::MATCH_C_ADD, fmt_cr_type),
            InstructionFilter::new("c.and", inst::MASK_C_AND, inst::MATCH_C_AND, fmt_ca_type),
            InstructionFilter::new("c.or", inst::MASK_C_OR, inst::MATCH_C_OR, fmt_ca_type),
            InstructionFilter::new("c.xor", inst::MASK_C_XOR, inst::MATCH_C_XOR, fmt_ca_type),
            InstructionFilter::new("c.sub", inst::MASK_C_SUB, inst::MATCH_C_SUB, fmt_ca_type),
            // `c.lui` must come after `c.addi16sp`, otherwise the `c.lui` filter will match any
            // `c.addi16sp` instructions.
            InstructionFilter::new(
                "c.lui",
                inst::MASK_C_LUI,
                inst::MATCH_C_LUI,
                fmt_ci_type_lui,
            ),
        ];

        pseudo_instruction_filters.extend(xlen_filters);
        pseudo_instruction_filters.extend(d_filters);
        pseudo_instruction_filters.extend(global_filters);
        pseudo_instruction_filters
    } else {
        vec![]
    };

    i_pseudo_instruction_inst_filters.extend(i_inst_filters);
    i_pseudo_instruction_inst_filters.extend(m_inst_filters);
    i_pseudo_instruction_inst_filters.extend(a_inst_filters);
    i_pseudo_instruction_inst_filters.extend(f_inst_filters);
    i_pseudo_instruction_inst_filters.extend(d_inst_filters);
    i_pseudo_instruction_inst_filters.extend(c_inst_filters);
    i_pseudo_instruction_inst_filters
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
