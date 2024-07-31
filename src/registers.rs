#[derive(Debug)]
pub struct Registers {
    regs: [u32; 32],
}

impl Registers {
    pub fn new_mips_default() -> Self {
        let mut regs: [u32; 32] = [0; 32];
        regs[28] = 0x10000000;
        regs[29] = 0x7FFFEFFC;

        Self { regs }
    }

    pub fn get(&self, index: u32) -> u32 {
        if index > 31 {
            panic!("Trying to access a register with index {index}")
        }

        self.regs[index as usize]
    }

    pub fn set(&mut self, index: u32, value: u32) {
        if index > 31 {
            panic!("Trying to access a register with index {index}")
        }

        if index != 0 {
            self.regs[index as usize] = value;
        }
    }

    pub fn dump(&self) -> [u32; 32] {
        self.regs
    }

    pub fn load(&mut self, regs: [u32; 32]) {
        self.regs = regs;
    }
}
