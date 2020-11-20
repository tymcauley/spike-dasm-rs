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
            (0x01f51513, "slli    a0, a0, 31"),
            (0x020005b7, "lui     a1, 0x2000"),
            (0x00000597, "auipc   a1, 0x0"),
            (0xffc62683, "lw      a3, -4(a2)"),
            (0x00d62023, "sw      a3, 0(a2)"),
            (0x03ff0a63, "beq     t5, t6, pc + 52"),
            (0x0ff0000f, "fence  "),
            (0x00000073, "ecall  "),
            (0x30200073, "mret   "),
            (0x10500073, "wfi    "),
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
}
