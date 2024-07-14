pub mod instruction;

type LC3Word = u16;
type LC3MemAddr = u16;
type LC3MemLoc = i16;

const ADDR_SPACE_SIZE:usize = 2_usize.pow(16_u32);
const TRAP_VEC_TBL:LC3Word = 0x0000;
const IR_VEC_TBL:LC3Word = 0x0100;
const OS_SUPER_STACK:LC3Word =  0x0200;
const USER_SPACE:LC3Word = 0x3000;
const DEV_REG_ADDR:LC3Word = 0xFE00;

pub struct ConditionReg {
    pub negative: LC3MemLoc,
    pub zero: LC3MemLoc,
    pub positive: LC3MemLoc,
}

pub struct RegFile {
    pub r0: LC3MemLoc,
    pub r1: LC3MemLoc,
    pub r2: LC3MemLoc,
    pub r3: LC3MemLoc,
    pub r4: LC3MemLoc,
    pub r5: LC3MemLoc,
    pub r6: LC3MemLoc,
    pub r7: LC3MemLoc,
}

pub struct LC3 {
    pub mem: Box<[LC3MemLoc; ADDR_SPACE_SIZE]>,
    pub conds: ConditionReg,
    pub regs: RegFile,
}

#[cfg(test)]
mod test {
    use super::instruction::*;

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
        let test_instr = IAnd::Reg(InstrReg{reg1:0b1001, reg2:0b0110});
        assert_eq!(test_instr.execute(), 0b0000)
    }

    #[test]
    fn test_not() {
        let test_instr = INot{reg:0b0000000000001001};
        assert_eq!(test_instr.execute(), 0b1111111111110110_u16 as i16)
    }
}
