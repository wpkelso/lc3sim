pub struct InstrImm {
    pub reg:i16,
    pub imm:i16,
}

pub struct InstrReg {
    pub reg1:i16,
    pub reg2:i16,
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
    pub reg:i16,
}


pub trait Instruction {
    type Output;

    fn execute(self) -> Self::Output;
}

impl Instruction for IAdd {
    type Output = i16;
    fn execute(self) -> Self::Output {
        match self {
            Self::Reg(InstrReg{reg1, reg2}) => reg1 + reg2,
            Self::Imm(InstrImm{reg, imm}) => reg + imm,
        }
    }
}

impl Instruction for IAnd {
    type Output = i16;
    fn execute(self) -> Self::Output {
        match self {
            Self::Reg(InstrReg{reg1, reg2}) => reg1 & reg2,
            Self::Imm(InstrImm{reg, imm}) => reg & imm,
        }
    }
}

impl Instruction for INot {
    type Output = i16;
    fn execute(self) -> Self::Output {
        !self.reg
    }
}
