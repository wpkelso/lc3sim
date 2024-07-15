pub mod instruction;

type LC3Word = u16;
type LC3MemAddr = u16;

const ADDR_SPACE_SIZE:usize = 2_usize.pow(16_u32);  //size of the memory address space
const TRAP_VEC_TBL:LC3Word = 0x0000;                //first address of the trap vector table
const IR_VEC_TBL:LC3Word = 0x0100;                  //first address of the interrupt vector table
const OS_SUPER_STACK:LC3Word =  0x0200;             //first address of the operating and supervisor
                                                        //stack space
const USER_SPACE:LC3Word = 0x3000;                  //first address of the user code space
const DEV_REG_ADDR:LC3Word = 0xFE00;                //first address of the device register address
                                                        //space

const NUM_REGS:usize = 8_usize;                        //number of registers in the LC3 spec

pub struct ConditionReg {
    pub negative: bool,
    pub zero: bool,
    pub positive: bool,
}

pub struct LC3 {
    pub mem: Box<[LC3Word; ADDR_SPACE_SIZE]>,
    pub conds: ConditionReg,
    pub regs: Box<[LC3Word; NUM_REGS]>,
    pub pc:LC3MemAddr, //program counter should be initialized on startup
}

#[cfg(test)]
mod test {
    use super::*;
    use super::instruction::*;

    #[test]
    fn test_add_imm() {
        let mut processor = LC3{
            mem: Box::new([0; ADDR_SPACE_SIZE]),
            conds: ConditionReg{negative: false, zero: false, positive: false},
            regs: Box::new([6, 4, 7, 10, 24, 8, 9, 18]),
            pc: 0x0000,
        };

        let test_instr = IAdd::Imm(InstrImm{dest_reg:1, src_reg:0, imm:5});
        test_instr.execute(&mut processor);
        assert_eq!(processor.regs[1], 11)
    }

    #[test]
    fn test_add_reg() {
        let mut processor = LC3{
            mem: Box::new([0; ADDR_SPACE_SIZE]),
            conds: ConditionReg{negative: false, zero: false, positive: false},
            regs: Box::new([6, 4, 7, 10, 24, 8, 9, 18]),
            pc: 0x0000,
        };

        let test_instr = IAdd::Reg(InstrReg{dest_reg: 1, src_reg_1: 0, src_reg_2: 3});
        test_instr.execute(&mut processor);
        assert_eq!(processor.regs[1], 16)
    }

    #[test]
    fn test_and_imm() {
        let mut processor = LC3{
            mem: Box::new([0; ADDR_SPACE_SIZE]),
            conds: ConditionReg{negative: false, zero: false, positive: false},
            regs: Box::new([6, 4, 7, 10, 24, 8, 9, 0]),
            pc: 0x0000,
        };

        let test_instr = IAnd::Imm(InstrImm{dest_reg: 1, src_reg: 0, imm: 0b0000000000000000});
        test_instr.execute(&mut processor);
        assert_eq!(processor.regs[1], 0b0000000000000000);
    }

    #[test]
    fn test_and_reg() {
        let mut processor = LC3{
            mem: Box::new([0; ADDR_SPACE_SIZE]),
            conds: ConditionReg{negative: false, zero: false, positive: false},
            regs: Box::new([6, 4, 7, 10, 24, 8, 9, 0]),
            pc: 0x0000,
        };

        let test_instr = IAnd::Reg(InstrReg{dest_reg: 1, src_reg_1: 0, src_reg_2: 7 });
        test_instr.execute(&mut processor);
        assert_eq!(processor.regs[1], 0b0000000000000000);
    }

    #[test]
    fn test_not() {
        let mut processor = LC3{
            mem: Box::new([0; ADDR_SPACE_SIZE]),
            conds: ConditionReg{negative: false, zero: false, positive: false},
            regs: Box::new([6, 4, 7, 10, 24, 8, 9, 0]),
            pc: 0x0000,
        };

        let test_instr = INot{dest_reg: 1, src_reg:7};
        test_instr.execute(&mut processor);
        assert_eq!(processor.regs[1], 0b1111111111111111);
    }

    #[test]
    fn test_not_conds() {
        let mut processor = LC3{
            mem: Box::new([0; ADDR_SPACE_SIZE]),
            conds: ConditionReg{negative: false, zero: false, positive: false},
            regs: Box::new([0b1111111111111111, 0, 0, 0b0000111100001111, 0, 0, 0, 0b1111000011110000]),
            pc: 0x0000,
        };

        //test positive
        let test_instr = INot{dest_reg: 1, src_reg:7};
        test_instr.execute(&mut processor);
        assert_eq!(processor.regs[1], 0b0000111100001111);
        assert_eq!(processor.conds.positive, true);
        assert_eq!(processor.conds.zero, false);
        assert_eq!(processor.conds.negative, false);

        //test zero
        let test_instr = INot{dest_reg: 1, src_reg:0};
        test_instr.execute(&mut processor);
        assert_eq!(processor.regs[1], 0b0000000000000000);
        assert_eq!(processor.conds.positive, false);
        assert_eq!(processor.conds.zero, true);
        assert_eq!(processor.conds.negative, false);

        //test negative
        let test_instr = INot{dest_reg: 1, src_reg:3};
        test_instr.execute(&mut processor);
        assert_eq!(processor.regs[1], 0b1111000011110000);
        assert_eq!(processor.conds.positive, false);
        assert_eq!(processor.conds.zero, false);
        assert_eq!(processor.conds.negative, true);
    }
}
