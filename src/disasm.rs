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
    fn disasm_rv64_pseudo_instructions() {
        let instructions = instruction::gen_instructions(Xlen::Rv64);
        let disasm = Disassembler::new(instructions);

        let test_pairs = vec![
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
        ];

        test_disasm(disasm, test_pairs);
    }
}
