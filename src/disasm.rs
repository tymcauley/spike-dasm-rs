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
    use crate::{Extensions, Xlen};

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
        let instructions = instruction::gen_instructions(Xlen::Rv64, Extensions::IMAFDC, true);
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
            (0x0135053b, "addw    a0, a0, s3"),
            (0x40e7853b, "subw    a0, a5, a4"),
            (0x0020973b, "sllw    a4, ra, sp"),
            (0x012c56bb, "srlw    a3, s8, s2"),
            (0x4020d73b, "sraw    a4, ra, sp"),
            (0x0100026f, "jal     tp, pc + 0x10"),
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
            (0xffc62683, "lw      a3, -4(a2)"),
            (0xff80e703, "lwu     a4, -8(ra)"),
            (0x00b0b283, "ld      t0, 11(ra)"),
            (0x001102a3, "sb      ra, 5(sp)"),
            (0x00111523, "sh      ra, 10(sp)"),
            (0x00d62023, "sw      a3, 0(a2)"),
            (0xfe20b423, "sd      sp, -24(ra)"),
            (0x0ff0000f, "fence"),
            (0x0000100f, "fence.i"),
            (0x00000073, "ecall"),
            (0x10500073, "wfi"),
            (0x30200073, "mret"),
            (0x12000073, "sfence.vma zero, zero"),
            (0x12050073, "sfence.vma a0, zero"),
        ];

        test_disasm(disasm, test_pairs);
    }

    #[test]
    fn disasm_rv64_csr() {
        let instructions = instruction::gen_instructions(Xlen::Rv64, Extensions::IMAFDC, true);
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
        let instructions = instruction::gen_instructions(Xlen::Rv64, Extensions::IMAFDC, true);
        let disasm = Disassembler::new(instructions);

        let test_pairs = vec![
            (0x00000013, "nop"),
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
            (0x00008067, "ret"),
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
        let instructions = instruction::gen_instructions(Xlen::Rv64, Extensions::IMAFDC, true);
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
        let instructions = instruction::gen_instructions(Xlen::Rv64, Extensions::IMAFDC, true);
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
        let instructions = instruction::gen_instructions(Xlen::Rv64, Extensions::IMAFDC, true);
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

    #[test]
    fn disasm_rv64_d() {
        let instructions = instruction::gen_instructions(Xlen::Rv64, Extensions::IMAFDC, true);
        let disasm = Disassembler::new(instructions);

        let test_pairs = vec![
            (0x01053107, "fld     ft2, 16(a0)"),
            (0x00233427, "fsd     ft2, 8(t1)"),
            (0x02e7f7d3, "fadd.d  fa5, fa5, fa4"),
            (0x0af67753, "fsub.d  fa4, fa2, fa5"),
            (0x12a7f7d3, "fmul.d  fa5, fa5, fa0"),
            (0x1a1071d3, "fdiv.d  ft3, ft0, ft1"),
            (0x5a0071d3, "fsqrt.d ft3, ft0"),
            (0x2a1001d3, "fmin.d  ft3, ft0, ft1"),
            (0x2a1011d3, "fmax.d  ft3, ft0, ft1"),
            (0x7ae6f7c3, "fmadd.d fa5, fa3, fa4, fa5"),
            (0x121071cf, "fnmadd.d ft3, ft0, ft1, ft2"),
            (0x121071c7, "fmsub.d ft3, ft0, ft1, ft2"),
            (0x121071cb, "fnmsub.d ft3, ft0, ft1, ft2"),
            (0xd20500d3, "fcvt.d.w ft1, a0"),
            (0xd2000053, "fcvt.d.w ft0, zero"),
            (0xd227f7d3, "fcvt.d.l fa5, a5"),
            (0xd2150053, "fcvt.d.wu ft0, a0"),
            (0xd2357053, "fcvt.d.lu ft0, a0"),
            (0xc200f0d3, "fcvt.w.d ra, ft1"),
            (0xc2201553, "fcvt.l.d a0, ft0"),
            (0xc2101553, "fcvt.wu.d a0, ft0"),
            (0xc2301553, "fcvt.lu.d a0, ft0"),
            (0x4011f1d3, "fcvt.s.d ft3, ft3"),
            (0x420001d3, "fcvt.d.s ft3, ft0"),
            (0x22208053, "fsgnj.d ft0, ft1, ft2"),
            (0x22209053, "fsgnjn.d ft0, ft1, ft2"),
            (0x22d6a6d3, "fsgnjx.d fa3, fa3, fa3"),
            (0xe2018553, "fmv.x.d a0, ft3"),
            (0xf20580d3, "fmv.d.x ft1, a1"),
            (0xa2102553, "feq.d   a0, ft0, ft1"),
            (0xa2101553, "flt.d   a0, ft0, ft1"),
            (0xa2100553, "fle.d   a0, ft0, ft1"),
            (0xe2051553, "fclass.d a0, fa0"),
        ];

        test_disasm(disasm, test_pairs);
    }

    #[test]
    fn disasm_rv64_c() {
        let instructions = instruction::gen_instructions(Xlen::Rv64, Extensions::IMAFDC, true);
        let disasm = Disassembler::new(instructions);

        let test_pairs = vec![
            (0x000045a2, "c.lwsp  a1, 8(sp)"),
            (0x000074a2, "c.ldsp  s1, 40(sp)"),
            (0x00002b6e, "c.fldsp fs6, 216(sp)"),
            (0x0000c62a, "c.swsp  a0, 12(sp)"),
            (0x0000c04e, "c.swsp  s3, 0(sp)"),
            (0x0000fc06, "c.sdsp  ra, 56(sp)"),
            (0x0000f822, "c.sdsp  s0, 48(sp)"),
            (0x0000e9ea, "c.sdsp  s10, 208(sp)"),
            (0x0000fdfe, "c.sdsp  t6, 248(sp)"),
            (0x0000a622, "c.fsdsp fs0, 264(sp)"),
            (0x0000a226, "c.fsdsp fs1, 256(sp)"),
            (0x0000bd6a, "c.fsdsp fs10, 184(sp)"),
            (0x0000439c, "c.lw    a5, 0(a5)"),
            (0x0000434c, "c.lw    a1, 4(a4)"),
            (0x00004808, "c.lw    a0, 16(s0)"),
            (0x00006094, "c.ld    a3, 0(s1)"),
            (0x00006494, "c.ld    a3, 8(s1)"),
            (0x00006890, "c.ld    a2, 16(s1)"),
            (0x00002198, "c.fld   fa4, 0(a1)"),
            (0x00002514, "c.fld   fa3, 8(a0)"),
            (0x000025f8, "c.fld   fa4, 200(a1)"),
            (0x0000c088, "c.sw    a0, 0(s1)"),
            (0x0000c1c8, "c.sw    a0, 4(a1)"),
            (0x0000c81c, "c.sw    a5, 16(s0)"),
            (0x0000e198, "c.sd    a4, 0(a1)"),
            (0x0000e51c, "c.sd    a5, 8(a0)"),
            (0x0000ee9c, "c.sd    a5, 24(a3)"),
            (0x0000a21c, "c.fsd   fa5, 0(a2)"),
            (0x0000aa1c, "c.fsd   fa5, 16(a2)"),
            (0x0000b118, "c.fsd   fa4, 32(a0)"),
            (0x0000bb1c, "c.fsd   fa5, 48(a4)"),
            (0x0000a001, "c.j     pc + 0"),
            (0x0000a021, "c.j     pc + 8"),
            (0x0000a029, "c.j     pc + 10"),
            (0x0000a809, "c.j     pc + 18"),
            (0x0000bfb1, "c.j     pc - 164"),
            (0x0000b5e9, "c.j     pc - 310"),
            (0x0000b54d, "c.j     pc - 350"),
            (0x0000b405, "c.j     pc - 1504"),
            (0x00008282, "c.jr    t0"),
            (0x00008782, "c.jr    a5"),
            (0x00009282, "c.jalr  t0"),
            (0x00009902, "c.jalr  s2"),
            (0x0000c119, "c.beqz  a0, pc + 6"),
            (0x0000dffd, "c.beqz  a5, pc - 2"),
            (0x0000d951, "c.beqz  a0, pc - 108"),
            (0x0000e925, "c.bnez  a0, pc + 112"),
            (0x0000ff89, "c.bnez  a5, pc - 230"),
            (0x00004781, "c.li    a5, 0"),
            (0x00004705, "c.li    a4, 1"),
            (0x00004729, "c.li    a4, 10"),
            (0x00004f61, "c.li    t5, 24"),
            (0x00005d7d, "c.li    s10, -1"),
            (0x000057c1, "c.li    a5, -16"),
            (0x00006405, "c.lui   s0, 0x1"),
            (0x000062f9, "c.lui   t0, 0x1e"),
            (0x00007405, "c.lui   s0, 0xfffe1"),
            (0x000076c1, "c.lui   a3, 0xffff0"),
            (0x00007afd, "c.lui   s5, 0xfffff"),
            (0x00000785, "c.addi  a5, 1"),
            (0x00000a91, "c.addi  s5, 4"),
            (0x000005d1, "c.addi  a1, 20"),
            (0x00001a71, "c.addi  s4, -4"),
            (0x00001541, "c.addi  a0, -16"),
            (0x00002781, "c.addiw a5, 0"),
            (0x00002509, "c.addiw a0, 2"),
            (0x0000367d, "c.addiw a2, -1"),
            (0x00006105, "c.addi16sp sp, 32"),
            (0x00006121, "c.addi16sp sp, 64"),
            (0x00006165, "c.addi16sp sp, 112"),
            (0x00006151, "c.addi16sp sp, 272"),
            (0x00006135, "c.addi16sp sp, 352"),
            (0x0000617d, "c.addi16sp sp, 496"),
            (0x00007139, "c.addi16sp sp, -64"),
            (0x0000716d, "c.addi16sp sp, -272"),
            (0x0000710d, "c.addi16sp sp, -352"),
            (0x00007101, "c.addi16sp sp, -512"),
            (0x00000000, "c.addi4spn s0, sp, 0"),
            (0x00000054, "c.addi4spn a3, sp, 4"),
            (0x0000002c, "c.addi4spn a1, sp, 8"),
            (0x00000980, "c.addi4spn s0, sp, 208"),
            (0x00001fe8, "c.addi4spn a0, sp, 1020"),
            (0x00000586, "c.slli  a1, 1"),
            (0x00000412, "c.slli  s0, 4"),
            (0x00000146, "c.slli  sp, 17"),
            (0x000002fe, "c.slli  t0, 31"),
            (0x000017fe, "c.slli  a5, 63"),
            (0x00008289, "c.srli  a3, 2"),
            (0x00008031, "c.srli  s0, 12"),
            (0x00008179, "c.srli  a0, 30"),
            (0x00008705, "c.srai  a4, 1"),
            (0x00008789, "c.srai  a5, 2"),
            (0x00008431, "c.srai  s0, 12"),
            (0x00008b8d, "c.andi  a5, 3"),
            (0x00008b9d, "c.andi  a5, 7"),
            (0x0000983d, "c.andi  s0, -17"),
            (0x00008522, "c.mv    a0, s0"),
            (0x000087f6, "c.mv    a5, t4"),
            (0x00009232, "c.add   tp, a2"),
            (0x000097de, "c.add   a5, s7"),
            (0x00008d65, "c.and   a0, s1"),
            (0x00008f6d, "c.and   a4, a1"),
            (0x00008fd9, "c.or    a5, a4"),
            (0x00008cc9, "c.or    s1, a0"),
            (0x00008ca9, "c.xor   s1, a0"),
            (0x00008f89, "c.sub   a5, a0"),
            (0x00008e05, "c.sub   a2, s1"),
            (0x00009f2d, "c.addw  a4, a1"),
            (0x00009d2d, "c.addw  a0, a1"),
            (0x00009ca9, "c.addw  s1, a0"),
            (0x00009f99, "c.subw  a5, a4"),
            (0x00009d01, "c.subw  a0, s0"),
            (0x00000001, "c.nop"),
            (0x00009002, "c.ebreak"),
            (0x00008082, "ret"),
        ];

        test_disasm(disasm, test_pairs);
    }

    #[test]
    fn disasm_rv32_c() {
        let instructions = instruction::gen_instructions(Xlen::Rv32, Extensions::IMAFDC, true);
        let disasm = Disassembler::new(instructions);

        let test_pairs = vec![
            (0x000065a2, "c.flwsp fa1, 8(sp)"),
            (0x0000fdfe, "c.fswsp ft11, 248(sp)"),
            (0x00006890, "c.flw   fa2, 16(s1)"),
            (0x0000e51c, "c.fsw   fa5, 8(a0)"),
            (0x00002001, "c.jal   pc + 0"),
            (0x00002021, "c.jal   pc + 8"),
            (0x00002029, "c.jal   pc + 10"),
            (0x00002809, "c.jal   pc + 18"),
            (0x00003fb1, "c.jal   pc - 164"),
            (0x000035e9, "c.jal   pc - 310"),
            (0x0000354d, "c.jal   pc - 350"),
            (0x00003405, "c.jal   pc - 1504"),
        ];

        test_disasm(disasm, test_pairs);
    }
}
