use crate::memory::Memory;
use crate::registers::Registers;

pub struct Core {
    pub(crate) memory: Memory,
    pub(crate) registers: Registers,
    pub(crate) pc: u32,
    hi: u32,
    lo: u32,
}

impl Core {
    pub fn new_mips_default() -> Self {
        Self {
            memory: Memory::new_mips_default(1024),
            registers: Registers::new_mips_default(),
            pc: 0x00100000,
            hi: 0,
            lo: 0,
        }
    }

    pub fn tick<F>(&mut self, syscall_handler: F)
    where
        F: FnMut(u32, [u32; 32]) -> [u32; 32],
    {
        // println!("PC={:#08X}", self.pc);

        // println!("$t2 = {:#04X}", self.registers.get(10));

        let inst = self.memory.get(self.pc).unwrap();

        // println!("Executing instruction {inst:#08X} at PC={:#08X}", self.pc);

        self.execute_instruction(inst, syscall_handler);

        self.pc += 1;
    }

    pub fn dump_registers(&self) -> [u32; 32] {
        self.registers.dump()
    }

    pub fn load_text(&mut self, text: Vec<u32>) {
        self.memory.load_text(text);
    }

    pub fn load_data(&mut self, data: Vec<u32>) {
        self.memory.load_data(data);
    }

    fn branch_with_offset(&mut self, mut offset: u32) {
        if offset & 0x00008000 != 0 {
            offset |= 0xFFFF0000;
            offset ^= 0xFFFFFFFF;
            offset += 1;
            self.pc -= offset;
        } else {
            self.pc += offset;
        }
    }
}

impl Core {
    pub(crate) fn execute_instruction<F>(&mut self, inst: u32, mut syscall_handler: F)
    where
        F: FnMut(u32, [u32; 32]) -> [u32; 32],
    {
        let opcode = (inst >> 26) & 0x3F;

        // If instruction is SYSCALL
        if opcode == 0x00 && (inst & 0x3F) == 0x0C {
            let new_regs = (syscall_handler)(inst, self.dump_registers());
            self.registers.load(new_regs);
            return;
        }

        // println!("{opcode:#04x}");

        match opcode {
            0x00 => self.execute_rtype(inst),
            0x02 => {
                // j
                let index = inst & 0x03FFFFFF;
                let target = (self.pc & 0xFC000000) | index;
                self.pc = target - 1; // Subtract 1 because we will add 1 in tick

                // println!("index={index:#08X}; New PC={:#08X}", self.pc);
            }
            0x04 => {
                // beq
                let (rs, rt, imm) = extract_itype_1(inst);

                let rs_val = self.registers.get(rs);
                let rt_val = self.registers.get(rt);
                if rs_val == rt_val {
                    self.branch_with_offset(imm);
                }
            }
            0x05 => {
                // bne
                let (rs, rt, imm) = extract_itype_1(inst);

                let rs_val = self.registers.get(rs);
                let rt_val = self.registers.get(rt);
                if rs_val != rt_val {
                    self.branch_with_offset(imm);
                }
            }
            0x08 => {
                // addi
                let (rs, rt, imm) = extract_itype_1(inst);

                let rs_val = self.registers.get(rs);
                self.registers.set(rt, rs_val + imm);
            }
            0x09 => {
                // addiu
                let (rs, rt, imm) = extract_itype_1(inst);

                let rs_val = self.registers.get(rs);
                self.registers.set(rt, rs_val + imm);
            }
            0x0A => {
                // slti
                let (rs, rt, imm) = extract_itype_1(inst);

                let rs_val = self.registers.get(rs);
                if rs_val < imm {
                    self.registers.set(rt, 1);
                } else {
                    self.registers.set(rt, 0);
                }
            }
            0x0C => {
                // andi
                let (rs, rt, imm) = extract_itype_1(inst);

                let rs_val = self.registers.get(rs);
                self.registers.set(rt, rs_val & imm);
            }
            0x0D => {
                // ori
                let (rs, rt, imm) = extract_itype_1(inst);

                let rs_val = self.registers.get(rs);
                self.registers.set(rt, rs_val | imm);
            }
            0x0E => {
                // xori
                let (rs, rt, imm) = extract_itype_1(inst);

                let rs_val = self.registers.get(rs);
                self.registers.set(rt, rs_val ^ imm);
            }
            0x0F => {
                // lui
                let (_rs, rt, imm) = extract_itype_1(inst);

                self.registers.set(rt, imm << 16);
            }
            _ => todo!("Unimplemented instruction: {:#04X}", inst),
        }
    }

    fn execute_rtype(&mut self, inst: u32) {
        let funct = inst & 0x3F;
        let shmt = (inst >> 6) & 0x1F;
        let rd = (inst >> 11) & 0x1F;
        let rt = (inst >> 16) & 0x1F;
        let rs = (inst >> 21) & 0x1F;

        let rt_val = self.registers.get(rt);
        let rs_val = self.registers.get(rs);

        match funct {
            0x00 => {
                // sll
                self.registers.set(rd, rt_val << shmt);
            }
            0x20 => {
                // add
                self.registers.set(rd, rt_val + rs_val);
            }
            0x21 => {
                // addu
                // println!(
                //     "{}: {}({:#04X}) + {}({:#04X})",
                //     self.pc - 0x00100000,
                //     rs,
                //     rs_val,
                //     rt,
                //     rt_val
                // );
                self.registers
                    .set(rd, ((rt_val as i32) + (rs_val as i32)) as u32);
            }
            0x24 => {
                // and
                self.registers.set(rd, rt_val & rs_val);
            }
            0x25 => {
                // or
                self.registers.set(rd, rt_val | rs_val);
            }
            0x26 => {
                // xor
                self.registers.set(rd, rt_val ^ rs_val);
            }
            0x2A => {
                // slt
                if rs_val < rt_val {
                    self.registers.set(rd, 0x01);
                } else {
                    self.registers.set(rd, 0x00);
                }
            }

            _ => {
                todo!("Unimplemented r-type instruction, funct: {:#04X}", funct)
            }
        }
    }

    // fn execute_addi(&mut self, inst: u32) {
    //     let (rs, rt, imm) = extract_itype_1(inst);

    //     let rs_val = self.registers.get(rs);
    //     self.registers.set(rt, rs_val + imm);
    // }
}

fn extract_itype_1(inst: u32) -> (u32, u32, u32) {
    ((inst >> 21) & 0x1F, (inst >> 16) & 0x1F, inst & 0x0000FFFF)
}

// #[cfg(tests)]
mod tests {
    use super::*;

    fn empty_syscall_fn(_: u32, _: [u32; 32]) -> [u32; 32] {
        [0; 32]
    }

    fn test_reg_reg(inst: u32, reg1: usize, reg2: usize) {
        let mut core = Core::new_mips_default();
        core.execute_instruction(inst, empty_syscall_fn);

        let regs = core.dump_registers();

        assert_eq!(regs[reg1], regs[reg2]);
    }

    fn test_reg_imm(inst: u32, reg1: usize, imm: u32) {
        let mut core = Core::new_mips_default();
        core.execute_instruction(inst, empty_syscall_fn);

        let regs = core.dump_registers();

        assert_eq!(regs[reg1], imm);
    }

    #[test]
    fn add_1() {
        test_reg_reg(0x03A84820, 9, 29);
    }

    #[test]
    fn addi_1() {
        test_reg_imm(0x200F002A, 15, 42);
    }

    #[test]
    fn addu_1() {}

    #[test]
    fn addiu_1() {}

    #[test]
    fn and_1() {}

    #[test]
    fn andi_1() {}
}
