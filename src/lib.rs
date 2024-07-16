pub mod instruction;

type LC3Word = u16;
type LC3MemAddr = u16;

const ADDR_SPACE_SIZE: usize = 2_usize.pow(16_u32); //size of the memory address space
const TRAP_VEC_TBL: LC3Word = 0x0000; //first address of the trap vector table
const IR_VEC_TBL: LC3Word = 0x0100; //first address of the interrupt vector table
const OS_SUPER_STACK: LC3Word = 0x0200; //first address of the operating and supervisor
                                        //stack space
const USER_SPACE: LC3Word = 0x3000; //first address of the user code space
const DEV_REG_ADDR: LC3Word = 0xFE00; //first address of the device register address
                                      //space

const NUM_REGS: usize = 8_usize; //number of registers in the LC3 spec

pub struct ConditionReg {
    pub negative: bool,
    pub zero: bool,
    pub positive: bool,
}

pub struct LC3 {
    pub mem: Box<[LC3Word; ADDR_SPACE_SIZE]>,
    pub conds: ConditionReg,
    pub regs: Box<[LC3Word; NUM_REGS]>,
    pub pc: LC3MemAddr, //program counter should be initialized on startup
}

#[cfg(test)]
mod test {
    use super::instruction::*;
    use super::*;

    //TODO: rewrite all of these in a more intelligent fashion
    #[test]
    fn instr_add_imm() {
        let mut processor = LC3 {
            mem: Box::new([0; ADDR_SPACE_SIZE]),
            conds: ConditionReg {
                negative: false,
                zero: false,
                positive: false,
            },
            regs: Box::new([6, 4, 7, 10, 24, 8, 9, 18]),
            pc: 0x0000,
        };

        let test_instr = IAdd::Imm(InstrRegImm {
            dest_reg: 1,
            src_reg: 0,
            imm: 5,
        });
        test_instr.execute(&mut processor);
        assert_eq!(processor.regs[1], 11)
    }

    #[test]
    fn instr_add_reg() {
        let mut processor = LC3 {
            mem: Box::new([0; ADDR_SPACE_SIZE]),
            conds: ConditionReg {
                negative: false,
                zero: false,
                positive: false,
            },
            regs: Box::new([6, 4, 7, 10, 24, 8, 9, 18]),
            pc: 0x0000,
        };

        let test_instr = IAdd::Reg(InstrRegReg {
            dest_reg: 1,
            src_reg_1: 0,
            src_reg_2: 3,
        });
        test_instr.execute(&mut processor);
        assert_eq!(processor.regs[1], 16)
    }

    #[test]
    fn instr_and_imm() {
        let mut processor = LC3 {
            mem: Box::new([0; ADDR_SPACE_SIZE]),
            conds: ConditionReg {
                negative: false,
                zero: false,
                positive: false,
            },
            regs: Box::new([6, 4, 7, 10, 24, 8, 9, 0]),
            pc: 0x0000,
        };

        let test_instr = IAnd::Imm(InstrRegImm {
            dest_reg: 1,
            src_reg: 0,
            imm: 0b0000000000000000,
        });
        test_instr.execute(&mut processor);
        assert_eq!(processor.regs[1], 0b0000000000000000);
    }

    #[test]
    fn instr_and_reg() {
        let mut processor = LC3 {
            mem: Box::new([0; ADDR_SPACE_SIZE]),
            conds: ConditionReg {
                negative: false,
                zero: false,
                positive: false,
            },
            regs: Box::new([6, 4, 7, 10, 24, 8, 9, 0]),
            pc: 0x0000,
        };

        let test_instr = IAnd::Reg(InstrRegReg {
            dest_reg: 1,
            src_reg_1: 0,
            src_reg_2: 7,
        });
        test_instr.execute(&mut processor);
        assert_eq!(processor.regs[1], 0b0000000000000000);
    }

    #[test]
    fn instr_not() {
        let mut processor = LC3 {
            mem: Box::new([0; ADDR_SPACE_SIZE]),
            conds: ConditionReg {
                negative: false,
                zero: false,
                positive: false,
            },
            regs: Box::new([
                0b0000000011111111,
                0b1111111100000000,
                0b0000111100001111,
                0b1111000011110000,
                0b0011001100110011,
                0b1100110011001100,
                0b0101010101010101,
                0b1010101010101010,
            ]),
            pc: 0x0000,
        };

        let test_instr = INot::Instr(InstrRegImm {
            dest_reg: 1,
            src_reg: 0,
            imm: 0,
        });
        test_instr.execute(&mut processor);
        assert_eq!(processor.regs[1], 0b1111111100000000);
    }

    #[test]
    fn instr_branch() {
        let mut processor = LC3 {
            mem: Box::new([0; ADDR_SPACE_SIZE]),
            conds: ConditionReg {
                negative: false,
                zero: false,
                positive: false,
            },
            regs: Box::new([0, 0, 0, 0, 0, 0, 0, 0]),
            pc: 0x0000,
        };

        //there are more clever ways to write this, I don't feel like writing them
        processor.conds = ConditionReg {
            positive: true,
            zero: false,
            negative: false,
        }; //test BRp
        let test_instr = IBranch {
            cond_codes: ConditionReg {
                positive: true,
                zero: false,
                negative: false,
            },
            pc_offset: 0x0002,
        };
        test_instr.execute(&mut processor);
        assert_eq!(processor.pc, 0x0002); //branch should've been taken
        processor.pc = 0x0000; //reset pc for next test

        processor.conds = ConditionReg {
            positive: false,
            zero: true,
            negative: false,
        }; //test BRz
        let test_instr = IBranch {
            cond_codes: ConditionReg {
                positive: false,
                zero: true,
                negative: false,
            },
            pc_offset: 0x0002,
        };
        test_instr.execute(&mut processor);
        assert_eq!(processor.pc, 0x0002); //branch should've been taken
        processor.pc = 0x0000; //reset pc for next test

        processor.conds = ConditionReg {
            positive: false,
            zero: false,
            negative: true,
        }; //test BRn
        let test_instr = IBranch {
            cond_codes: ConditionReg {
                positive: false,
                zero: false,
                negative: true,
            },
            pc_offset: 0x0002,
        };
        test_instr.execute(&mut processor);
        assert_eq!(processor.pc, 0x0002); //branch should've been taken
        processor.pc = 0x0000; //reset pc for next test

        processor.conds = ConditionReg {
            positive: true,
            zero: true,
            negative: false,
        }; //test BRpz
        let test_instr = IBranch {
            cond_codes: ConditionReg {
                positive: true,
                zero: true,
                negative: false,
            },
            pc_offset: 0x0002,
        };
        test_instr.execute(&mut processor);
        assert_eq!(processor.pc, 0x0002); //branch should've been taken
        processor.pc = 0x0000; //reset pc for next test

        processor.conds = ConditionReg {
            positive: true,
            zero: false,
            negative: true,
        }; //test BRpn
        let test_instr = IBranch {
            cond_codes: ConditionReg {
                positive: true,
                zero: false,
                negative: true,
            },
            pc_offset: 0x0002,
        };
        test_instr.execute(&mut processor);
        assert_eq!(processor.pc, 0x0002); //branch should've been taken
        processor.pc = 0x0000; //reset pc for next test

        processor.conds = ConditionReg {
            positive: false,
            zero: true,
            negative: true,
        }; //test BRzn
        let test_instr = IBranch {
            cond_codes: ConditionReg {
                positive: false,
                zero: true,
                negative: true,
            },
            pc_offset: 0x0002,
        };
        test_instr.execute(&mut processor);
        assert_eq!(processor.pc, 0x0002); //branch should've been taken
        processor.pc = 0x0000; //reset pc for next test

        processor.conds = ConditionReg {
            positive: true,
            zero: true,
            negative: true,
        }; //test BRzn
        let test_instr = IBranch {
            cond_codes: ConditionReg {
                positive: true,
                zero: true,
                negative: true,
            },
            pc_offset: 0x0002,
        };
        test_instr.execute(&mut processor);
        assert_eq!(processor.pc, 0x0002); //branch should've been taken
        processor.pc = 0x0000; //reset pc for next test
    }

    #[test]
    fn instr_jmp() {
        let mut processor = LC3 {
            mem: Box::new([0; ADDR_SPACE_SIZE]),
            conds: ConditionReg {
                negative: false,
                zero: false,
                positive: false,
            },
            regs: Box::new([
                0x3000, 0x0000, 0x1000, 0x0200, 0xff00, 0xfe00, 0x3000, 0x7301,
            ]),
            pc: 0x0000,
        };

        let mut i = 0;
        while i < 8 {
            let test_instr = IJump::Instr(InstrOffset6 {
                base_reg: i,
                target_reg: 0, //unused
                offset: 0,     //unused
            });
            test_instr.execute(&mut processor);
            assert_eq!(processor.pc, processor.regs[i as usize]);
            i += 1;
        }
    }

    #[test]
    fn instr_jsr() {
        let mut processor = LC3 {
            mem: Box::new([0; ADDR_SPACE_SIZE]),
            conds: ConditionReg {
                negative: false,
                zero: false,
                positive: false,
            },
            regs: Box::new([
                0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
            ]),
            pc: 0x3000,
        };

        let test_instr = IJumpSubRoutine::Offset(InstrPCOffset11 { pc_offset: 0x0006 });
        test_instr.execute(&mut processor);
        assert_eq!(processor.pc, 0x3006);
        assert_eq!(processor.regs[7], 0x3000);

        processor.pc = 0x3000;
        processor.regs[1] = 0x000A;
        processor.regs[7] = 0x0000;

        let test_instr = IJumpSubRoutine::Reg(InstrOffset6 {
            target_reg: 0, //unused
            base_reg: 1,
            offset: 0, //unused
        });
        test_instr.execute(&mut processor);
        assert_eq!(processor.pc, 0x000A);
        assert_eq!(processor.regs[7], 0x3000);
    }
}
