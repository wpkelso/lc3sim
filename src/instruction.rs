//TODO: TRAP instructions

use crate::{
    defs::{LC3Word, RegAddr, SignedLC3Word},
    executors::LC3,
};

pub trait Instruction {
    fn execute<P: LC3>(self, processor: &mut P);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct InstrRegImm {
    pub dest_reg: RegAddr,
    pub src_reg: RegAddr,
    pub imm: LC3Word,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct InstrRegReg {
    pub dest_reg: RegAddr,
    pub src_reg_1: RegAddr,
    pub src_reg_2: RegAddr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct InstrOffset6 {
    pub target_reg: RegAddr,
    pub base_reg: RegAddr,
    pub offset: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct InstrPCOffset9 {
    pub target_reg: RegAddr,
    pub pc_offset: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct InstrPCOffset11 {
    pub pc_offset: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IAdd {
    Reg(InstrRegReg),
    Imm(InstrRegImm),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IAnd {
    Reg(InstrRegReg),
    Imm(InstrRegImm),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum INot {
    Instr(InstrRegImm), //not actually a RegImm, just used for implementation
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConditionCodes {
    pub positive: bool,
    pub negative: bool,
    pub zero: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct IBranch {
    //while br roughly follows the bit assignment of PCoffset9,
    //this is treated as a special case for ease of implementation
    pub cond_codes: ConditionCodes,
    pub pc_offset: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IJump {
    Instr(InstrOffset6), //not strictly an offset6, but doesn't matter here
    Ret, //RET and RETI are included here, as they are functionally special cases of JMP
    InterRet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IJumpSubRoutine {
    Offset(InstrPCOffset11), //JSR
    Reg(InstrOffset6),       //JSRR treated as an offset6 with an offset of 0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ILoad {
    Std(InstrPCOffset9),      //LD
    Indirect(InstrPCOffset9), //LDI
    Reg(InstrOffset6),        //LDR
    Addr(InstrPCOffset9),     //LEA
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IStore {
    Std(InstrPCOffset9),      //ST
    Indirect(InstrPCOffset9), //STI
    Reg(InstrOffset6),        //STR
}

/// Set the processor condition codes from `result`.
fn set_condition_codes<P: LC3>(processor: &mut P, result: LC3Word) {
    match (result as SignedLC3Word).cmp(&0) {
        std::cmp::Ordering::Greater => processor.flag_positive(),
        std::cmp::Ordering::Less => processor.flag_negative(),
        std::cmp::Ordering::Equal => processor.flag_zero(),
    }
}

impl Instruction for IAdd {
    fn execute<P: LC3>(self, processor: &mut P) {
        let dest;
        let result = match self {
            Self::Reg(InstrRegReg {
                dest_reg,
                src_reg_1,
                src_reg_2,
            }) => {
                dest = dest_reg;
                processor.reg(src_reg_1) + processor.reg(src_reg_2)
            }
            Self::Imm(InstrRegImm {
                dest_reg,
                src_reg,
                imm,
            }) => {
                dest = dest_reg;
                processor.reg(src_reg) + imm
            }
        };
        processor.set_reg(dest, result);
        set_condition_codes(processor, result);
    }
}

impl Instruction for IAnd {
    fn execute<P: LC3>(self, processor: &mut P) {
        let dest;
        let result = match self {
            Self::Reg(InstrRegReg {
                dest_reg,
                src_reg_1,
                src_reg_2,
            }) => {
                dest = dest_reg;
                processor.reg(src_reg_1) & processor.reg(src_reg_2)
            }
            Self::Imm(InstrRegImm {
                dest_reg,
                src_reg,
                imm,
            }) => {
                dest = dest_reg;
                processor.reg(src_reg) & imm
            }
        };
        processor.set_reg(dest, result);
        set_condition_codes(processor, result);
    }
}

impl Instruction for INot {
    fn execute<P: LC3>(self, processor: &mut P) {
        let dest;
        let result = match self {
            Self::Instr(InstrRegImm {
                dest_reg, src_reg, ..
            }) => {
                dest = dest_reg;
                !processor.reg(src_reg)
            }
        };
        processor.set_reg(dest, result);
        set_condition_codes(processor, result);
    }
}

impl Instruction for IBranch {
    fn execute<P: LC3>(self, processor: &mut P) {
        let pos_condition = self.cond_codes.positive && processor.positive_cond();
        let zero_condition = self.cond_codes.zero && processor.zero_cond();
        let neg_condition = self.cond_codes.negative && processor.negative_cond();

        if pos_condition || zero_condition || neg_condition {
            processor.set_pc(processor.pc() + self.pc_offset);
        }
    }
}

impl Instruction for IJump {
    fn execute<P: LC3>(self, processor: &mut P) {
        let dest;
        match self {
            Self::Instr(InstrOffset6 { base_reg, .. }) => {
                dest = base_reg;
            }
            Self::Ret => {
                dest = RegAddr::Seven;
            }
            Self::InterRet => {
                unimplemented!();
            }
        }
        processor.set_pc(processor.reg(dest));
    }
}

impl Instruction for IJumpSubRoutine {
    fn execute<P: LC3>(self, processor: &mut P) {
        processor.set_reg(RegAddr::Seven, processor.pc()); //save return address
        let jump_addr: u16;
        match self {
            Self::Offset(InstrPCOffset11 { pc_offset }) => {
                //JSR
                jump_addr = processor.pc() + pc_offset;
            }
            Self::Reg(InstrOffset6 { base_reg, .. }) => {
                //JSRR
                jump_addr = processor.reg(base_reg);
            }
        };
        processor.set_pc(jump_addr);
    }
}

impl Instruction for ILoad {
    fn execute<P: LC3>(self, processor: &mut P) {
        let result;
        match self {
            Self::Std(InstrPCOffset9 {
                target_reg,
                pc_offset,
            }) => {
                let target_addr: u16 = processor.pc() + 1 + pc_offset;
                result = processor.mem(target_addr);
                processor.set_reg(target_reg, result);
            }
            Self::Indirect(InstrPCOffset9 {
                target_reg,
                pc_offset,
            }) => {
                let target_addr: u16 = processor.pc() + 1 + pc_offset;
                let target_loc: u16 = processor.mem(target_addr);
                result = processor.mem(target_loc);
                processor.set_reg(target_reg, result);
            }
            Self::Reg(InstrOffset6 {
                target_reg,
                base_reg,
                offset,
            }) => {
                let target_addr = processor.reg(base_reg) + offset;
                result = processor.mem(target_addr);
                processor.set_reg(target_reg, result);
            }
            Self::Addr(InstrPCOffset9 {
                target_reg,
                pc_offset,
            }) => {
                result = processor.pc() + 1 + pc_offset;
                processor.set_reg(target_reg, result);
            }
        }

        set_condition_codes(processor, result);
    }
}

impl Instruction for IStore {
    fn execute<P: LC3>(self, processor: &mut P) {
        match self {
            Self::Std(InstrPCOffset9 {
                target_reg,
                pc_offset,
            }) => {
                let target_addr: u16 = processor.pc() + 1 + pc_offset;
                processor.set_mem(target_addr, processor.reg(target_reg));
            }
            Self::Indirect(InstrPCOffset9 {
                target_reg,
                pc_offset,
            }) => {
                let calc_addr: u16 = processor.pc() + 1 + pc_offset;
                let target_addr: u16 = processor.mem(calc_addr);
                processor.set_mem(target_addr, processor.reg(target_reg));
            }
            Self::Reg(InstrOffset6 {
                target_reg,
                base_reg,
                offset,
            }) => {
                let target_addr: u16 = processor.reg(base_reg) + offset;
                processor.set_mem(target_addr, processor.reg(target_reg));
            }
        }
    }
}
