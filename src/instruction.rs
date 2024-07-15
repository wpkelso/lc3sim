use crate::*;

pub struct InstrImm {
    pub dest_reg:u8,
    pub src_reg:u8,
    pub imm:u16,
}

pub struct InstrReg {
    pub dest_reg:u8,
    pub src_reg_1:u8,
    pub src_reg_2:u8,
}

pub enum IAdd {
    Reg(InstrReg),
    Imm(InstrImm),
}

pub enum IAnd {
    Reg(InstrReg),
    Imm(InstrImm),
}

pub struct INot {
    pub dest_reg:u8,
    pub src_reg:u8,
}


pub trait Instruction {
    fn execute(self, processor:&mut LC3);
}

impl Instruction for IAdd {
    fn execute(self, processor:&mut LC3) {
        let dest;
        let result = match self {
            Self::Reg(InstrReg{dest_reg, src_reg_1, src_reg_2}) => {
                dest = dest_reg;
                processor.regs[src_reg_1 as usize] + processor.regs[src_reg_2 as usize]
            },
            Self::Imm(InstrImm{dest_reg, src_reg, imm}) => {
                dest = dest_reg;
                processor.regs[src_reg as usize] + imm
            },
        };
        processor.regs[dest as usize] = result;

        if (result as i16) > 0 {
            processor.conds.positive = true;
            processor.conds.zero = false;
            processor.conds.negative = false;
        } else if (result as i16) < 0 {
            processor.conds.positive = false;
            processor.conds.zero = false;
            processor.conds.negative = true;
        } else {
            processor.conds.positive = false;
            processor.conds.zero = true;
            processor.conds.negative = false;
        }
    }
}

impl Instruction for IAnd {
    fn execute(self, processor:&mut LC3) {
        let dest;
        let result = match self {
            Self::Reg(InstrReg{dest_reg, src_reg_1, src_reg_2}) => {
                dest = dest_reg;
                processor.regs[src_reg_1 as usize] & processor.regs[src_reg_2 as usize]
            },
            Self::Imm(InstrImm{dest_reg, src_reg, imm}) => {
                dest = dest_reg;
                processor.regs[src_reg as usize] & imm
            },
        };
        processor.regs[dest as usize] = result;

        if (result as i16) > 0 {
            processor.conds.positive = true;
            processor.conds.zero = false;
            processor.conds.negative = false;
        } else if (result as i16) < 0 {
            processor.conds.positive = false;
            processor.conds.zero = false;
            processor.conds.negative = true;
        } else {
            processor.conds.positive = false;
            processor.conds.zero = true;
            processor.conds.negative = false;
        }
    }
}

impl Instruction for INot {
    fn execute(self, processor:&mut LC3) {
        let result = !processor.regs[self.src_reg as usize];
        processor.regs[self.dest_reg as usize] = result;

        if (result as i16) > 0 {
            processor.conds.positive = true;
            processor.conds.zero = false;
            processor.conds.negative = false;
        } else if (result as i16) < 0 {
            processor.conds.positive = false;
            processor.conds.zero = false;
            processor.conds.negative = true;
        } else {
            processor.conds.positive = false;
            processor.conds.zero = true;
            processor.conds.negative = false;
        }
    }
}
