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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_add_imm() {
        let test_instr = IAdd::Imm(InstrImm{reg:10, imm:5});
        assert_eq!(test_instr.execute(), 15)
    }

    #[test]
    fn test_add_reg() {
        let test_instr = IAdd::Reg(InstrReg{reg1:10, reg2:5});
        assert_eq!(test_instr.execute(), 15)
    }

    #[test]
    fn test_and_imm() {
        let test_instr = IAnd::Imm(InstrImm{reg:0b1001, imm: 0b0110});
        assert_eq!(test_instr.execute(), 0b0000)
    }

    #[test]
    fn test_and_reg() {
        let test_instr = IAnd::Reg(InstrReg{reg:0b1001, imm:0b1001});
    }
}
