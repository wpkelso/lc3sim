use crate::*;

pub trait Instruction {
    fn execute(self, processor: &mut LC3);
}

pub struct InstrRegImm {
    pub dest_reg: u8,
    pub src_reg: u8,
    pub imm: u16,
}

pub struct InstrRegReg {
    pub dest_reg: u8,
    pub src_reg_1: u8,
    pub src_reg_2: u8,
}

pub struct InstrOffset6 {
    pub target_reg: u8,
    pub base_reg: u8,
    pub offset: u16,
}

pub struct InstrPCOffset9 {
    pub target_reg: u8,
    pub pc_offset: u16,
}

pub struct InstrPCOffset11 {
    pub pc_offset: u16,
}

pub enum IAdd {
    Reg(InstrRegReg),
    Imm(InstrRegImm),
}

pub enum IAnd {
    Reg(InstrRegReg),
    Imm(InstrRegImm),
}

pub enum INot {
    Instr(InstrRegImm), //not actually a RegImm, just used for implementation
}

pub struct IBranch {
    //while br roughly follows the bit assignment of PCoffset9,
    //this is treated as a special case for ease of implementation
    pub cond_codes: ConditionReg,
    pub pc_offset: u16,
}

pub enum IJump {
    Instr(InstrOffset6), //not strictly an offset6, but doesn't matter here
}

pub enum IJumpSubRoutine {
    Offset(InstrPCOffset11), //JSR
    Reg(InstrOffset6),       //JSRR treated as an offset6 with an offset of 0
}

pub enum ISubRoutineReturn {
    Instr(InstrOffset6), //this could be hardcoded, but it's clearer to follow the machine code
                         //convention
}

pub enum IInterruptReturn {
    Instr(InstrOffset6), //treated as an offset6 with an offset of 0
}

pub enum ILoad {
    Std(InstrPCOffset9),      //LD
    Indirect(InstrPCOffset9), //LDI
    Reg(InstrOffset6),        //LDR
    Addr(InstrPCOffset9),     //LEA
}

pub enum IStore {
    Std(InstrPCOffset9),      //ST
    Indirect(InstrPCOffset9), //STI
    Reg(InstrOffset6),        //STR
}

impl Instruction for IAdd {
    fn execute(self, processor: &mut LC3) {
        let dest;
        let result = match self {
            Self::Reg(InstrRegReg {
                dest_reg,
                src_reg_1,
                src_reg_2,
            }) => {
                dest = dest_reg;
                processor.regs[src_reg_1 as usize] + processor.regs[src_reg_2 as usize]
            }
            Self::Imm(InstrRegImm {
                dest_reg,
                src_reg,
                imm,
            }) => {
                dest = dest_reg;
                processor.regs[src_reg as usize] + imm
            }
        };
        processor.regs[dest as usize] = result;

        if (result as i16) > 0 {
            processor.conds = ConditionReg {
                positive: true,
                zero: false,
                negative: false,
            };
        } else if (result as i16) < 0 {
            processor.conds = ConditionReg {
                positive: false,
                zero: false,
                negative: true,
            };
        } else {
            processor.conds = ConditionReg {
                positive: false,
                zero: true,
                negative: false,
            };
        }
    }
}

impl Instruction for IAnd {
    fn execute(self, processor: &mut LC3) {
        let dest;
        let result = match self {
            Self::Reg(InstrRegReg {
                dest_reg,
                src_reg_1,
                src_reg_2,
            }) => {
                dest = dest_reg;
                processor.regs[src_reg_1 as usize] & processor.regs[src_reg_2 as usize]
            }
            Self::Imm(InstrRegImm {
                dest_reg,
                src_reg,
                imm,
            }) => {
                dest = dest_reg;
                processor.regs[src_reg as usize] & imm
            }
        };
        processor.regs[dest as usize] = result;

        if (result as i16) > 0 {
            processor.conds = ConditionReg {
                positive: true,
                zero: false,
                negative: false,
            };
        } else if (result as i16) < 0 {
            processor.conds = ConditionReg {
                positive: false,
                zero: false,
                negative: true,
            };
        } else {
            processor.conds = ConditionReg {
                positive: false,
                zero: true,
                negative: false,
            };
        }
    }
}

impl Instruction for INot {
    fn execute(self, processor: &mut LC3) {
        let dest;
        let result = match self {
            Self::Instr(InstrRegImm {
                dest_reg, src_reg, ..
            }) => {
                dest = dest_reg;
                !processor.regs[src_reg as usize]
            }
        };
        processor.regs[dest as usize] = result;
        if (result as i16) > 0 {
            processor.conds = ConditionReg {
                positive: true,
                zero: false,
                negative: false,
            };
        } else if (result as i16) < 0 {
            processor.conds = ConditionReg {
                positive: false,
                zero: false,
                negative: true,
            };
        } else {
            processor.conds = ConditionReg {
                positive: false,
                zero: true,
                negative: false,
            };
        }
    }
}

impl Instruction for IBranch {
    fn execute(self, processor: &mut LC3) {
        if (self.cond_codes.positive && processor.conds.positive)
            || (self.cond_codes.zero && processor.conds.zero)
            || (self.cond_codes.negative && processor.conds.negative)
        {
            processor.pc += self.pc_offset;
        }
    }
}

impl Instruction for IJump {
    fn execute(self, processor: &mut LC3) {
        let dest;
        match self {
            Self::Instr(InstrOffset6 { base_reg, .. }) => {
                dest = base_reg;
            }
        }
        processor.pc = processor.regs[dest as usize];
    }
}

impl Instruction for IJumpSubRoutine {
    fn execute(self, processor: &mut LC3) {
        processor.regs[7] = processor.pc;
        let jump_addr: u16;
        match self {
            Self::Offset(InstrPCOffset11 { pc_offset }) => {
                //JSR
                jump_addr = processor.pc + pc_offset;
            }
            Self::Reg(InstrOffset6 { base_reg, .. }) => {
                //JSRR
                jump_addr = processor.regs[base_reg as usize];
            }
        };
        processor.pc = jump_addr;
    }
}

impl Instruction for ISubRoutineReturn {
    fn execute(self, processor: &mut LC3) {
        todo!();
    }
}

impl Instruction for IInterruptReturn {
    fn execute(self, processor: &mut LC3) {
        todo!();
    }
}

impl Instruction for ILoad {
    fn execute(self, processor: &mut LC3) {
        match self {
            Self::Std(InstrPCOffset9 {
                target_reg,
                pc_offset,
            }) => {
                let target_addr: u16 = processor.pc + 1 + pc_offset;
                processor.regs[target_reg as usize] = processor.mem[target_addr as usize];
            }
            Self::Indirect(InstrPCOffset9 {
                target_reg,
                pc_offset,
            }) => {
                let target_addr: u16 = processor.pc + 1 + pc_offset;
                let target_loc: u16 = processor.mem[target_addr as usize];
                processor.regs[target_reg as usize] = processor.mem[target_loc as usize];
            }
            Self::Reg(InstrOffset6 {
                target_reg,
                base_reg,
                offset,
            }) => {
                let target_addr: u16 = processor.regs[base_reg as usize] + offset as u16;
                processor.regs[target_reg as usize] = processor.mem[target_addr as usize];
            }
            Self::Addr(InstrPCOffset9 {
                target_reg,
                pc_offset,
            }) => {
                let target_addr = processor.pc + 1 + pc_offset;
                processor.regs[target_reg as usize] = target_addr;
            }
        }
    }
}

impl Instruction for IStore {
    fn execute(self, processor: &mut LC3) {
        match self {
            Self::Std(InstrPCOffset9 {
                target_reg,
                pc_offset,
            }) => {
                todo!()
            }
            Self::Indirect(InstrPCOffset9 {
                target_reg,
                pc_offset,
            }) => {
                todo!()
            }
            Self::Reg(InstrOffset6 {
                target_reg,
                base_reg,
                offset,
            }) => {
                todo!()
            }
        }
    }
}
