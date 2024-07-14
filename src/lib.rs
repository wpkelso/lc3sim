mod byte_conv;

use std::fmt::Debug;

use enum_dispatch::enum_dispatch;

pub type AsmLine = u16;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct InstrImm {
    pub dr: u8,
    pub reg: u8,
    pub imm: i16,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct InstrReg {
    pub dr: u8,
    pub reg1: u8,
    pub reg2: u8,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum IAdd {
    Reg(InstrReg),
    Imm(InstrImm),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum IAnd {
    Reg(InstrReg),
    Imm(InstrImm),
}

/// Some LC-3 instruction.
#[enum_dispatch]
pub trait Instruction: TryFrom<AsmLine> + Into<AsmLine> {
    fn execute(self);
}

/// Enum with all [`Instruction`] implementors.
#[enum_dispatch(Instruction)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum InstEnum {
    IAdd,
    IAnd,
}

impl Instruction for IAdd {
    fn execute(self) {
        unimplemented!()
    }
}

impl Instruction for IAnd {
    fn execute(self) {
        unimplemented!()
    }
}
