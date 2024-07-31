#[derive(Debug)]
pub struct Memory {
    pub(crate) text: Vec<u32>,
    pub(crate) data: Vec<u32>,
    blocksize: usize,
    text_start: u32,
    text_end: u32,
    data_start: u32,
    data_end: u32,

    pub(crate) text_generation: u32,
    pub(crate) data_generation: u32,
    blocks_allocated: usize,
}

impl Memory {
    pub fn new_mips_default(blocksize: usize) -> Self {
        let mut mem = Self {
            text: Vec::with_capacity(blocksize),
            data: Vec::with_capacity(blocksize),
            blocksize,
            text_start: 0x00100000, // 0x00400000 / 4
            text_end: 0x04003FFF,   // 0x10010000 / 4 - 1
            data_start: 0x04004000, // 0x10010000 / 4
            data_end: 0x1FFFFFFF,   // 0x80000000 / 4 - 1

            text_generation: 0,
            data_generation: 0,
            blocks_allocated: 2,
        };

        for _ in 0..blocksize {
            mem.text.push(0);
            mem.data.push(0);
        }

        mem
    }

    pub fn load_text(&mut self, text: Vec<u32>) {
        for i in 0..text.len() {
            self.set(self.text_start + i as u32, text[i]);
        }
    }

    pub fn load_data(&mut self, data: Vec<u32>) {
        for i in 0..data.len() {
            self.set(self.data_start + i as u32, data[i]);
        }
    }

    pub fn get(&self, index: u32) -> Option<u32> {
        if self.text_start <= index && index <= self.text_end {
            let index = index - self.text_start;

            if index as usize >= self.text.len() {
                return Some(0);
            }

            return match self.text.get(index as usize) {
                Some(val) => Some(*val),
                None => None,
            };
        }

        if self.data_start <= index && index <= self.data_end {
            let index = index - self.data_start;

            if index as usize >= self.data.len() {
                return Some(0);
            }

            return match self.data.get(index as usize) {
                Some(val) => Some(*val),
                None => None,
            };
        }

        None
    }

    pub fn set(&mut self, index: u32, value: u32) {
        if self.text_start <= index && index <= self.text_end {
            let index = index - self.text_start;

            while index as usize >= self.text.len() {
                self.text.reserve(self.blocksize);
                self.blocks_allocated += 1;
                for _ in 0..self.blocksize {
                    self.text.push(0);
                }
            }

            self.text[index as usize] = value;
            self.text_generation += 1;
        }

        if self.data_start <= index && index <= self.data_end {
            let index = index - self.data_start;

            while index as usize >= self.data.len() {
                self.data.reserve(self.blocksize);
                self.blocks_allocated += 1;
                for _ in 0..self.blocksize {
                    self.data.push(0);
                }
            }

            self.data[index as usize] = value;
            self.data_generation += 1;
        }
    }
}

// #[derive(Debug)]
// struct Memory {
//     mem: HashMap<u32, u32>,
//     low: u32,
//     high: u32,
// }

// impl Memory {
//     pub fn new() -> Self {
//         Self {
//             mem: HashMap::new(),
//             low: 0x00400000,
//             high: 0x7fffffff,
//         }
//     }

//     pub fn get(&self, index: u32) -> Option<u32> {
//         if index < self.low || index > self.high {
//             return None;
//         }

//         if self.mem.contains_key(&index) {
//             return Some(self.mem[&index]);
//         }

//         Some(0)
//     }

//     pub fn set(&mut self, index: u32, value: u32) {
//         self.mem.insert(index, value);
//     }
// }

// impl Index<u32> for Memory {
//     type Output = u32;

//     fn index(&self, index: u32) -> &Self::Output {}
// }
// impl IndexMut<u32> for Memory {

// }
