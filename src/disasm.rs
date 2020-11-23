use super::instruction::{InstructionBits, InstructionFilter};

pub struct Disassembler {
    instructions: Vec<InstructionFilter>,
}

impl Disassembler {
    pub fn new(instructions: Vec<InstructionFilter>) -> Self {
        Self { instructions }
    }

    fn get_inst(&self, x: InstructionBits) -> Option<&InstructionFilter> {
        self.instructions.iter().find(|inst| inst.is_eq(x))
    }

    pub fn fmt_inst(&self, x: InstructionBits) -> Option<String> {
        self.get_inst(x)
            .map(|inst_filter| (inst_filter.formatter)(inst_filter, x))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::instruction;
    use crate::Xlen;

    fn test_disasm(disasm: Disassembler, test_pairs: Vec<(u32, &str)>) {
        for (inst_u32, inst_str) in test_pairs.into_iter() {
            let inst_bits = InstructionBits::new(inst_u32).unwrap_or_else(|e| {
                panic!(
                    "Failed to construct InstructionBits from {:0>8x} ({}): {}",
                    inst_u32, inst_str, e
                );
            });

            assert_eq!(
                disasm.fmt_inst(inst_bits).unwrap_or_else(|| panic!(
                    "{:?} didn't match any known instruction, from {:0>8x}, '{}'",
                    inst_bits, inst_u32, inst_str
                )),
                inst_str
            );
        }
    }

    #[test]
    fn disasm_rv64_simple() {
        let instructions = instruction::gen_instructions(Xlen::Rv64);
        let disasm = Disassembler::new(instructions);

        let test_pairs = vec![
            (0xfc050513, "addi    a0, a0, -64"),
            (0xf0008093, "addi    ra, ra, -256"),
            (0x8000a713, "slti    a4, ra, -2048"),
            (0x00d0b093, "sltiu   ra, ra, 13"),
            (0x0f00e713, "ori     a4, ra, 240"),
            (0xf0f0c713, "xori    a4, ra, -241"),
            (0x70f0f713, "andi    a4, ra, 1807"),
            (0xfff00e9b, "addiw   t4, zero, -1"),
            (0x01f51513, "slli    a0, a0, 31"),
            (0x00e0d713, "srli    a4, ra, 14"),
            (0x40405093, "srai    ra, zero, 4"),
            (0x00e0971b, "slliw   a4, ra, 14"),
            (0x00e0d71b, "srliw   a4, ra, 14"),
            (0x4070d71b, "sraiw   a4, ra, 7"),
            (0x020005b7, "lui     a1, 0x2000"),
            (0x00000597, "auipc   a1, 0x0"),
            (0x00208733, "add     a4, ra, sp"),
            (0x00102133, "slt     sp, zero, ra"),
            (0x0020b733, "sltu    a4, ra, sp"),
            (0x0020f733, "and     a4, ra, sp"),
            (0x0020e733, "or      a4, ra, sp"),
            (0x0020c033, "xor     zero, ra, sp"),
            (0x00209733, "sll     a4, ra, sp"),
            (0x00105133, "srl     sp, zero, ra"),
            (0x40208733, "sub     a4, ra, sp"),
            (0x40105133, "sra     sp, zero, ra"),
            (0x0100026f, "jal     tp, pc + 0x10"),
            (0xffc62683, "lw      a3, -4(a2)"),
            (0x000306e7, "jalr    a3, t1, 0"),
            (0x03ff0a63, "beq     t5, t6, pc + 52"),
            (0xfc521ee3, "bne     tp, t0, pc - 36"),
            (0x0420c063, "blt     ra, sp, pc + 64"),
            (0x2620e263, "bltu    ra, sp, pc + 612"),
            (0x0a20da63, "bge     ra, sp, pc + 180"),
            (0xfe20fee3, "bgeu    ra, sp, pc - 4"),
            (0xffd08703, "lb      a4, -3(ra)"),
            (0x0000c703, "lbu     a4, 0(ra)"),
            (0x00a11703, "lh      a4, 10(sp)"),
            (0xffe0d703, "lhu     a4, -2(ra)"),
            (0x0040a703, "lw      a4, 4(ra)"),
            (0xff80e703, "lwu     a4, -8(ra)"),
            (0x00b0b283, "ld      t0, 11(ra)"),
            (0x001102a3, "sb      ra, 5(sp)"),
            (0x00111523, "sh      ra, 10(sp)"),
            (0x00d62023, "sw      a3, 0(a2)"),
            (0xfe20b423, "sd      sp, -24(ra)"),
            (0x0ff0000f, "fence  "),
            (0x0000100f, "fence.i"),
            (0x00000073, "ecall  "),
            (0x10500073, "wfi    "),
            (0x30200073, "mret   "),
        ];

        test_disasm(disasm, test_pairs);
    }

    #[test]
    fn disasm_rv64_csr() {
        let instructions = instruction::gen_instructions(Xlen::Rv64);
        let disasm = Disassembler::new(instructions);

        let test_pairs = vec![
            (0x18031573, "csrrw   a0, satp, t1"),
            (0x18032573, "csrrs   a0, satp, t1"),
            (0x18033573, "csrrc   a0, satp, t1"),
            (0x18035573, "csrrwi  a0, satp, 6"),
            (0x18036573, "csrrsi  a0, satp, 6"),
            (0x18037573, "csrrci  a0, satp, 6"),
        ];

        test_disasm(disasm, test_pairs);
    }

    #[test]
    fn disasm_rv64_pseudo_instructions() {
        let instructions = instruction::gen_instructions(Xlen::Rv64);
        let disasm = Disassembler::new(instructions);

        let test_pairs = vec![
            (0x00000013, "nop    "),
            (0x00300193, "li      gp, 3"),
            (0x00010513, "mv      a0, sp"),
            (0xfff74813, "not     a6, a4"),
            (0x0005851b, "sext.w  a0, a1"),
            (0x0017b793, "seqz    a5, a5"),
            (0x00103133, "snez    sp, ra"),
            (0xffdff06f, "j       pc - 0x4"),
            (0x04c0006f, "j       pc + 0x4c"),
            (0x7ea0106f, "j       pc + 0x17ea"),
            (0x269020ef, "jal     pc + 0x2a68"),
            (0xe48ff0ef, "jal     pc - 0x9b8"),
            (0x00008067, "ret    "),
            (0x000f0067, "jr      t5"),
            (0x000300e7, "jalr    t1"),
            (0x00050463, "beqz    a0, pc + 8"),
            (0xfe0816e3, "bnez    a6, pc - 20"),
            (0x0e09c563, "bltz    s3, pc + 234"),
            (0x0002d863, "bgez    t0, pc + 16"),
            (0x300027f3, "csrr    a5, mstatus"),
            (0x3b061073, "csrw    pmpaddr0, a2"),
            (0x3412a073, "csrs    mepc, t0"),
            (0x30063073, "csrc    mstatus, a2"),
            (0x30415073, "csrwi   mie, 2"),
            (0x30046073, "csrsi   mstatus, 8"),
            (0x30127073, "csrci   misa, 4"),
        ];

        test_disasm(disasm, test_pairs);
    }

    #[test]
    fn disasm_rv64_m() {
        let instructions = instruction::gen_instructions(Xlen::Rv64);
        let disasm = Disassembler::new(instructions);

        let test_pairs = vec![
            (0x02208733, "mul     a4, ra, sp"),
            (0x02101133, "mulh    sp, zero, ra"),
            (0x0220b733, "mulhu   a4, ra, sp"),
            (0x0220a733, "mulhsu  a4, ra, sp"),
            (0x03678b3b, "mulw    s6, a5, s6"),
            (0x0220c733, "div     a4, ra, sp"),
            (0x03b4ddb3, "divu    s11, s1, s11"),
            (0x0220e733, "rem     a4, ra, sp"),
            (0x02e7f9b3, "remu    s3, a5, a4"),
            (0x037b473b, "divw    a4, s6, s7"),
            (0x0220d73b, "divuw   a4, ra, sp"),
            (0x0220e73b, "remw    a4, ra, sp"),
            (0x0220f73b, "remuw   a4, ra, sp"),
        ];

        test_disasm(disasm, test_pairs);
    }

    #[test]
    fn disasm_rv64_a() {
        let instructions = instruction::gen_instructions(Xlen::Rv64);
        let disasm = Disassembler::new(instructions);

        let test_pairs = vec![
            (0x1005272f, "lr.w    a4, (a0)"),
            (0x1005372f, "lr.d    a4, (a0)"),
            (0x18e5272f, "sc.w    a4, a4, (a0)"),
            (0x18e5372f, "sc.d    a4, a4, (a0)"),
            (0x08b6a72f, "amoswap.w a4, a1, (a3)"),
            (0x08b6b72f, "amoswap.d a4, a1, (a3)"),
            (0x00b5262f, "amoadd.w a2, a1, (a0)"),
            (0x00b6b72f, "amoadd.d a4, a1, (a3)"),
            (0x60b6a72f, "amoand.w a4, a1, (a3)"),
            (0x60b6b72f, "amoand.d a4, a1, (a3)"),
            (0x40b6a72f, "amoor.w a4, a1, (a3)"),
            (0x40b6b72f, "amoor.d a4, a1, (a3)"),
            (0x20b6a72f, "amoxor.w a4, a1, (a3)"),
            (0x20b6b72f, "amoxor.d a4, a1, (a3)"),
            (0xa0b6a72f, "amomax.w a4, a1, (a3)"),
            (0xa0b6b72f, "amomax.d a4, a1, (a3)"),
            (0xe0b6a72f, "amomaxu.w a4, a1, (a3)"),
            (0xe0b6b72f, "amomaxu.d a4, a1, (a3)"),
            (0x80b6a72f, "amomin.w a4, a1, (a3)"),
            (0x80b6b72f, "amomin.d a4, a1, (a3)"),
            (0xc0b6a72f, "amominu.w a4, a1, (a3)"),
            (0xc0b6b72f, "amominu.d a4, a1, (a3)"),
        ];

        test_disasm(disasm, test_pairs);
    }

    #[test]
    fn disasm_rv64_f() {
        let instructions = instruction::gen_instructions(Xlen::Rv64);
        let disasm = Disassembler::new(instructions);

        let test_pairs = vec![
            (0xe7c52007, "flw     ft0, -388(a0)"),
            (0x00452087, "flw     ft1, 4(a0)"),
            (0x00052007, "flw     ft0, 0(a0)"),
            (0x0005a027, "fsw     ft0, 0(a1)"),
            (0x0015aa27, "fsw     ft1, 20(a1)"),
            (0x00242827, "fsw     ft2, 16(s0)"),
            (0x001071d3, "fadd.s  ft3, ft0, ft1"),
            (0x081071d3, "fsub.s  ft3, ft0, ft1"),
            (0x10107053, "fmul.s  ft0, ft0, ft1"),
            (0x181071d3, "fdiv.s  ft3, ft0, ft1"),
            (0x580071d3, "fsqrt.s ft3, ft0"),
            (0x281001d3, "fmin.s  ft3, ft0, ft1"),
            (0x281011d3, "fmax.s  ft3, ft0, ft1"),
            (0x101071c3, "fmadd.s ft3, ft0, ft1, ft2"),
            (0x101071cf, "fnmadd.s ft3, ft0, ft1, ft2"),
            (0x101071c7, "fmsub.s ft3, ft0, ft1, ft2"),
            (0x101071cb, "fnmsub.s ft3, ft0, ft1, ft2"),
            (0xd0057053, "fcvt.s.w ft0, a0"),
            (0xd0257053, "fcvt.s.l ft0, a0"),
            (0xd0157053, "fcvt.s.wu ft0, a0"),
            (0xd0357053, "fcvt.s.lu ft0, a0"),
            (0xc000f0d3, "fcvt.w.s ra, ft1"),
            (0xc020f0d3, "fcvt.l.s ra, ft1"),
            (0xc0101553, "fcvt.wu.s a0, ft0"),
            (0xc0301553, "fcvt.lu.s a0, ft0"),
            (0x201081d3, "fsgnj.s ft3, ft1, ft1"),
            (0x20209053, "fsgnjn.s ft0, ft1, ft2"),
            (0x2020a053, "fsgnjx.s ft0, ft1, ft2"),
            (0xf00003d3, "fmv.w.x ft7, zero"),
            (0xf0060153, "fmv.w.x ft2, a2"),
            (0xf0060153, "fmv.w.x ft2, a2"),
            (0xe0000553, "fmv.x.w a0, ft0"),
            (0xa0102553, "feq.s   a0, ft0, ft1"),
            (0xa0101553, "flt.s   a0, ft0, ft1"),
            (0xa0100553, "fle.s   a0, ft0, ft1"),
            (0xe0051553, "fclass.s a0, fa0"),
        ];

        test_disasm(disasm, test_pairs);
    }
}
