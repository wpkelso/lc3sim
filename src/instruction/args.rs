use crate::defs::{LC3Word, RegAddr, SignedLC3Word};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstrRegSignedImm {
    pub dest_reg: RegAddr,
    pub src_reg: RegAddr,
    pub imm: SignedLC3Word,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstrRegImm {
    pub dest_reg: RegAddr,
    pub src_reg: RegAddr,
    pub imm: LC3Word,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstrRegReg {
    pub dest_reg: RegAddr,
    pub src_reg_1: RegAddr,
    pub src_reg_2: RegAddr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstrRegOnly {
    pub dest_reg: RegAddr,
    pub src_reg: RegAddr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstrOffset6 {
    pub target_reg: RegAddr,
    pub base_reg: RegAddr,
    pub offset: SignedLC3Word,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstrPCOffset9 {
    pub target_reg: RegAddr,
    pub pc_offset: SignedLC3Word,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstrPCOffset11 {
    pub pc_offset: SignedLC3Word,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConditionCodes {
    pub positive: bool,
    pub negative: bool,
    pub zero: bool,
}
