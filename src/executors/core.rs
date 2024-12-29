use std::iter::FusedIterator;

use crate::{
    defs::{
        LC3MemAddr, LC3Word, RegAddr, ADDR_SPACE_SIZE, NUM_REGS, STACK_REG, SUPERVISOR_SP_INIT,
    },
    instruction::Instruction,
};

use super::{LC3MemLoc, StepFailure, LC3};

#[derive(Debug, Clone, Copy)]
struct ConditionReg {
    pub negative: bool,
    pub zero: bool,
    pub positive: bool,
}

#[derive(Debug, Clone)]
pub struct CoreLC3 {
    mem: Box<[LC3Word; ADDR_SPACE_SIZE]>,
    conds: ConditionReg,
    priority: u8,
    privileged: bool,
    regs: Box<[LC3Word; NUM_REGS]>,
    supervisor_sp: LC3Word,
    pc: LC3MemAddr,
}

impl CoreLC3 {
    pub fn new() -> Self {
        Self {
            mem: Box::new([0; ADDR_SPACE_SIZE]),
            conds: ConditionReg {
                negative: false,
                zero: false,
                positive: false,
            },
            priority: 0,
            privileged: false,
            supervisor_sp: SUPERVISOR_SP_INIT,
            regs: Box::new([0; NUM_REGS]),
            pc: 0x0000,
        }
    }
}

impl Default for CoreLC3 {
    fn default() -> Self {
        Self::new()
    }
}

impl LC3 for CoreLC3 {
    fn pc(&self) -> LC3MemAddr {
        self.pc
    }
    fn set_pc(&mut self, pc: LC3MemAddr) {
        self.pc = pc
    }

    fn reg(&self, addr: RegAddr) -> LC3Word {
        let reg_addr = usize::from(addr);

        if self.privileged && reg_addr == STACK_REG.into() {
            self.supervisor_sp
        } else {
            self.regs[reg_addr]
        }
    }
    fn set_reg(&mut self, addr: RegAddr, value: LC3Word) {
        let reg_addr = usize::from(addr);

        if self.privileged && reg_addr == STACK_REG.into() {
            self.supervisor_sp = value
        } else {
            self.regs[reg_addr] = value
        }
    }

    fn mem(&self, addr: LC3MemAddr) -> LC3Word {
        self.mem[addr as usize]
    }
    fn set_mem(&mut self, addr: LC3MemAddr, value: LC3Word) {
        self.mem[addr as usize] = value;
    }

    fn priority(&self) -> u8 {
        self.priority
    }
    fn set_priority(&mut self, priority: u8) {
        if priority < 8 {
            self.priority = priority
        }
    }

    fn privileged(&self) -> bool {
        self.privileged
    }
    fn set_privileged(&mut self, priviledged: bool) {
        self.privileged = priviledged
    }

    fn positive_cond(&self) -> bool {
        self.conds.positive
    }
    fn zero_cond(&self) -> bool {
        self.conds.zero
    }
    fn negative_cond(&self) -> bool {
        self.conds.negative
    }

    fn flag_positive(&mut self) {
        self.conds = ConditionReg {
            negative: false,
            zero: false,
            positive: true,
        }
    }
    fn flag_zero(&mut self) {
        self.conds = ConditionReg {
            negative: false,
            zero: true,
            positive: false,
        }
    }
    fn flag_negative(&mut self) {
        self.conds = ConditionReg {
            negative: true,
            zero: false,
            positive: false,
        }
    }

    fn clear_flags(&mut self) {
        self.conds = ConditionReg {
            negative: false,
            zero: false,
            positive: false,
        }
    }

    type FullIter<'a> = std::iter::Cloned<std::slice::Iter<'a, LC3Word>>;
    fn iter(&self) -> Self::FullIter<'_> {
        self.mem.iter().cloned()
    }

    type SparseIter<'a> = CoreLC3SparseIter<'a>;
    fn sparse_iter(&self) -> Self::SparseIter<'_> {
        CoreLC3SparseIter::new(self.iter())
    }

    fn halt(&mut self) {
        todo!()
    }

    fn unhalt(&mut self) {
        todo!()
    }

    fn is_halted(&self) -> bool {
        todo!()
    }

    /// Executes the current instruction.
    ///
    /// Does not handle memory map updates.
    fn step(&mut self) -> Result<(), StepFailure> {
        let inst = self
            .cur_inst()
            .ok_or(StepFailure::InvalidInstruction(self.mem(self.pc())))?;

        Ok(inst.execute(self)?)
    }

    fn populate<I: IntoIterator<Item = LC3Word>>(&mut self, start: LC3MemAddr, words: I) {
        let mem_iter_mut = self.mem[start.into()..].iter_mut();
        for (word, loc) in words.into_iter().zip(mem_iter_mut) {
            *loc = word;
        }
    }
}

/// Sparse iterator for [`CoreLC3`].
///
/// Skips all nonzero elements.
pub struct CoreLC3SparseIter<'a> {
    iter: std::iter::Enumerate<<CoreLC3 as LC3>::FullIter<'a>>,
}

impl<'a> CoreLC3SparseIter<'a> {
    fn new(iter: <CoreLC3 as LC3>::FullIter<'a>) -> Self {
        Self {
            iter: iter.enumerate(),
        }
    }
}

impl Iterator for CoreLC3SparseIter<'_> {
    type Item = LC3MemLoc;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (loc, value) = self.iter.next()?;
            if value != 0 {
                return Some(LC3MemLoc {
                    loc: loc as u16,
                    value,
                });
            }
        }
    }
}

impl DoubleEndedIterator for CoreLC3SparseIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            let (loc, value) = self.iter.next_back()?;
            if value != 0 {
                return Some(LC3MemLoc {
                    loc: loc as u16,
                    value,
                });
            }
        }
    }
}

impl FusedIterator for CoreLC3SparseIter<'_> {}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{defs::RegAddr, instruction::*};

    //TODO: rewrite all of these in a more intelligent fashion
    #[test]
    fn instr_add_imm() {
        let mut processor = CoreLC3 {
            mem: Box::new([0; ADDR_SPACE_SIZE]),
            conds: ConditionReg {
                negative: false,
                zero: false,
                positive: false,
            },
            priority: 0,
            privileged: false,
            supervisor_sp: SUPERVISOR_SP_INIT,
            regs: Box::new([6, 4, 7, 10, 24, 8, 9, 18]),
            pc: 0x0000,
        };

        let test_instr = IAdd::Imm(InstrRegImm {
            dest_reg: const { RegAddr::panic_from_u8(1) },
            src_reg: const { RegAddr::panic_from_u8(0) },
            imm: 5,
        });
        test_instr.execute(&mut processor).unwrap();
        assert_eq!(processor.regs[1], 11)
    }

    #[test]
    fn instr_add_reg() {
        let mut processor = CoreLC3 {
            mem: Box::new([0; ADDR_SPACE_SIZE]),
            conds: ConditionReg {
                negative: false,
                zero: false,
                positive: false,
            },
            priority: 0,
            privileged: false,
            supervisor_sp: SUPERVISOR_SP_INIT,
            regs: Box::new([6, 4, 7, 10, 24, 8, 9, 18]),
            pc: 0x0000,
        };

        let test_instr = IAdd::Reg(InstrRegReg {
            dest_reg: const { RegAddr::panic_from_u8(1) },
            src_reg_1: const { RegAddr::panic_from_u8(0) },
            src_reg_2: const { RegAddr::panic_from_u8(3) },
        });
        test_instr.execute(&mut processor).unwrap();
        assert_eq!(processor.regs[1], 16)
    }

    #[test]
    fn instr_and_imm() {
        let mut processor = CoreLC3 {
            mem: Box::new([0; ADDR_SPACE_SIZE]),
            conds: ConditionReg {
                negative: false,
                zero: false,
                positive: false,
            },
            priority: 0,
            privileged: false,
            supervisor_sp: SUPERVISOR_SP_INIT,
            regs: Box::new([6, 4, 7, 10, 24, 8, 9, 0]),
            pc: 0x0000,
        };

        let test_instr = IAnd::Imm(InstrRegImm {
            dest_reg: const { RegAddr::panic_from_u8(1) },
            src_reg: const { RegAddr::panic_from_u8(0) },
            imm: 0b0000000000000000,
        });
        test_instr.execute(&mut processor).unwrap();
        assert_eq!(processor.regs[1], 0b0000000000000000);
    }

    #[test]
    fn instr_and_reg() {
        let mut processor = CoreLC3 {
            mem: Box::new([0; ADDR_SPACE_SIZE]),
            conds: ConditionReg {
                negative: false,
                zero: false,
                positive: false,
            },
            priority: 0,
            privileged: false,
            supervisor_sp: SUPERVISOR_SP_INIT,
            regs: Box::new([6, 4, 7, 10, 24, 8, 9, 0]),
            pc: 0x0000,
        };

        let test_instr = IAnd::Reg(InstrRegReg {
            dest_reg: const { RegAddr::panic_from_u8(1) },
            src_reg_1: const { RegAddr::panic_from_u8(0) },
            src_reg_2: const { RegAddr::panic_from_u8(7) },
        });
        test_instr.execute(&mut processor).unwrap();
        assert_eq!(processor.regs[1], 0b0000000000000000);
    }

    #[test]
    fn instr_not() {
        let mut processor = CoreLC3 {
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
            priority: 0,
            privileged: false,
            supervisor_sp: SUPERVISOR_SP_INIT,
            pc: 0x0000,
        };

        let test_instr = INot(InstrRegOnly {
            dest_reg: const { RegAddr::panic_from_u8(1) },
            src_reg: const { RegAddr::panic_from_u8(0) },
        });
        test_instr.execute(&mut processor).unwrap();
        assert_eq!(processor.regs[1], 0b1111111100000000);
    }

    #[test]
    fn instr_branch() {
        let mut processor = CoreLC3 {
            mem: Box::new([0; ADDR_SPACE_SIZE]),
            conds: ConditionReg {
                negative: false,
                zero: false,
                positive: false,
            },
            priority: 0,
            privileged: false,
            supervisor_sp: SUPERVISOR_SP_INIT,
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
            cond_codes: ConditionCodes {
                positive: true,
                zero: false,
                negative: false,
            },
            pc_offset: 0x0002,
        };
        test_instr.execute(&mut processor).unwrap();
        assert_eq!(processor.pc, 0x0002); //branch should've been taken
        processor.pc = 0x0000; //reset pc for next test

        processor.conds = ConditionReg {
            positive: false,
            zero: true,
            negative: false,
        }; //test BRz
        let test_instr = IBranch {
            cond_codes: ConditionCodes {
                positive: false,
                zero: true,
                negative: false,
            },
            pc_offset: 0x0002,
        };
        test_instr.execute(&mut processor).unwrap();
        assert_eq!(processor.pc, 0x0002); //branch should've been taken
        processor.pc = 0x0000; //reset pc for next test

        processor.conds = ConditionReg {
            positive: false,
            zero: false,
            negative: true,
        }; //test BRn
        let test_instr = IBranch {
            cond_codes: ConditionCodes {
                positive: false,
                zero: false,
                negative: true,
            },
            pc_offset: 0x0002,
        };
        test_instr.execute(&mut processor).unwrap();
        assert_eq!(processor.pc, 0x0002); //branch should've been taken
        processor.pc = 0x0000; //reset pc for next test

        processor.conds = ConditionReg {
            positive: true,
            zero: true,
            negative: false,
        }; //test BRpz
        let test_instr = IBranch {
            cond_codes: ConditionCodes {
                positive: true,
                zero: true,
                negative: false,
            },
            pc_offset: 0x0002,
        };
        test_instr.execute(&mut processor).unwrap();
        assert_eq!(processor.pc, 0x0002); //branch should've been taken
        processor.pc = 0x0000; //reset pc for next test

        processor.conds = ConditionReg {
            positive: true,
            zero: false,
            negative: true,
        }; //test BRpn
        let test_instr = IBranch {
            cond_codes: ConditionCodes {
                positive: true,
                zero: false,
                negative: true,
            },
            pc_offset: 0x0002,
        };
        test_instr.execute(&mut processor).unwrap();
        assert_eq!(processor.pc, 0x0002); //branch should've been taken
        processor.pc = 0x0000; //reset pc for next test

        processor.conds = ConditionReg {
            positive: false,
            zero: true,
            negative: true,
        }; //test BRzn
        let test_instr = IBranch {
            cond_codes: ConditionCodes {
                positive: false,
                zero: true,
                negative: true,
            },
            pc_offset: 0x0002,
        };
        test_instr.execute(&mut processor).unwrap();
        assert_eq!(processor.pc, 0x0002); //branch should've been taken
        processor.pc = 0x0000; //reset pc for next test

        processor.conds = ConditionReg {
            positive: true,
            zero: true,
            negative: true,
        }; //test BRzn
        let test_instr = IBranch {
            cond_codes: ConditionCodes {
                positive: true,
                zero: true,
                negative: true,
            },
            pc_offset: 0x0002,
        };
        test_instr.execute(&mut processor).unwrap();
        assert_eq!(processor.pc, 0x0002); //branch should've been taken
        processor.pc = 0x0000; //reset pc for next test
    }

    #[test]
    fn instr_jmp() {
        let mut processor = CoreLC3 {
            mem: Box::new([0; ADDR_SPACE_SIZE]),
            conds: ConditionReg {
                negative: false,
                zero: false,
                positive: false,
            },
            priority: 0,
            privileged: false,
            supervisor_sp: SUPERVISOR_SP_INIT,
            regs: Box::new([
                0x3000, 0x0000, 0x1000, 0x0200, 0xff00, 0xfe00, 0x3000, 0x7301,
            ]),
            pc: 0x0000,
        };

        for i in 0..8 {
            let test_instr = IJump::Instr(RegAddr::panic_from_u8(i));
            test_instr.execute(&mut processor).unwrap();
            assert_eq!(processor.pc, processor.regs[i as usize]);
        }

        processor.regs[7] = 0x3000;
        let test_instr: IJump = IJump::Ret;
        test_instr.execute(&mut processor).unwrap();
        assert_eq!(processor.pc, 0x3000);
    }

    #[test]
    fn instr_jsr() {
        let mut processor = CoreLC3 {
            mem: Box::new([0; ADDR_SPACE_SIZE]),
            conds: ConditionReg {
                negative: false,
                zero: false,
                positive: false,
            },
            priority: 0,
            privileged: false,
            supervisor_sp: SUPERVISOR_SP_INIT,
            regs: Box::new([
                0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
            ]),
            pc: 0x3000,
        };

        // JSR
        let test_instr = IJumpSubRoutine::Offset(InstrPCOffset11 { pc_offset: 0x0006 });
        test_instr.execute(&mut processor).unwrap();
        assert_eq!(processor.pc, 0x3006);
        assert_eq!(processor.regs[7], 0x3000);

        processor.pc = 0x3000;
        processor.regs[1] = 0x000A;
        processor.regs[7] = 0x0000;

        // JSRR
        let test_instr = IJumpSubRoutine::Reg(RegAddr::One);
        test_instr.execute(&mut processor).unwrap();
        assert_eq!(processor.pc, 0x000A);
        assert_eq!(processor.regs[7], 0x3000);
    }

    #[test]
    fn instr_ld() {
        let mut processor: CoreLC3 = CoreLC3 {
            mem: Box::new([0; ADDR_SPACE_SIZE]),
            conds: ConditionReg {
                negative: false,
                zero: false,
                positive: false,
            },
            priority: 0,
            privileged: false,
            supervisor_sp: SUPERVISOR_SP_INIT,
            regs: Box::new([
                0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
            ]),
            pc: 0x3000,
        };
        let test_instr: ILoad = ILoad::Std(InstrPCOffset9 {
            target_reg: const { RegAddr::panic_from_u8(0) },
            pc_offset: 0x0005,
        });

        //LD
        processor.mem[0x3006] = 0xFF14;
        test_instr.execute(&mut processor).unwrap();
        assert_eq!(processor.regs[0], 0xFF14);

        // LDI
        processor.mem[0x3003] = 0x3004;
        processor.mem[0x3004] = 0xFF14;
        let test_instr: ILoad = ILoad::Indirect(InstrPCOffset9 {
            target_reg: const { RegAddr::panic_from_u8(1) },
            pc_offset: 0x0002,
        });
        test_instr.execute(&mut processor).unwrap();
        assert_eq!(processor.regs[1], 0xFF14);

        // LDR
        processor.mem[0x300A] = 0xFF14;
        processor.regs[2] = 0x3009;
        let test_instr: ILoad = ILoad::Reg(InstrOffset6 {
            target_reg: const { RegAddr::panic_from_u8(3) },
            base_reg: const { RegAddr::panic_from_u8(2) },
            offset: 0x0001,
        });
        test_instr.execute(&mut processor).unwrap();
        assert_eq!(processor.regs[3], 0xFF14);

        // LEA
        let test_instr: ILoad = ILoad::Addr(InstrPCOffset9 {
            target_reg: const { RegAddr::panic_from_u8(4) },
            pc_offset: 0x000E,
        });
        test_instr.execute(&mut processor).unwrap();
        assert_eq!(processor.regs[4], 0x300F);
    }

    #[test]
    fn instr_st() {
        let mut processor: CoreLC3 = CoreLC3 {
            mem: Box::new([0; ADDR_SPACE_SIZE]),
            conds: ConditionReg {
                negative: false,
                zero: false,
                positive: false,
            },
            priority: 0,
            privileged: false,
            supervisor_sp: SUPERVISOR_SP_INIT,
            regs: Box::new([
                0xFF14, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
            ]),
            pc: 0x3000,
        };

        // ST
        let test_instr: IStore = IStore::Std(InstrPCOffset9 {
            target_reg: const { RegAddr::panic_from_u8(0) },
            pc_offset: 0x0004,
        });
        test_instr.execute(&mut processor).unwrap();
        assert_eq!(processor.mem[0x3005], 0xFF14);

        // STI
        processor.mem[0x3003] = 0x300A;
        let test_instr: IStore = IStore::Indirect(InstrPCOffset9 {
            target_reg: const { RegAddr::panic_from_u8(0) },
            pc_offset: 0x0002,
        });
        test_instr.execute(&mut processor).unwrap();
        assert_eq!(processor.mem[0x300A], 0xFF14);

        // STR
        processor.regs[1] = 0x3003;
        let test_instr: IStore = IStore::Reg(InstrOffset6 {
            target_reg: const { RegAddr::panic_from_u8(0) },
            base_reg: const { RegAddr::panic_from_u8(1) },
            offset: 0x0003,
        });
        test_instr.execute(&mut processor).unwrap();
        assert_eq!(processor.mem[0x3006], 0xFF14);
    }

    #[test]
    fn priority_reg() {
        let mut processor = CoreLC3::new();

        processor.set_priority(3);
        assert_eq!(processor.priority(), 3);
        processor.set_priority(0);
        assert_eq!(processor.priority(), 0);
    }

    #[test]
    fn privilege_reg() {
        let mut processor = CoreLC3::new();

        processor.set_privileged(true);
        assert!(processor.privileged());
        processor.set_privileged(false);
        assert!(!processor.privileged());
    }

    #[test]
    fn processor_status_reg() {
        let mut processor = CoreLC3::new();

        processor.set_priority(0);
        processor.set_privileged(true);
        processor.clear_flags();

        assert_eq!(processor.processor_status_reg(), 0);

        processor.set_privileged(false);
        assert_eq!(processor.processor_status_reg(), 0x8000);

        processor.flag_negative();
        assert_eq!(processor.processor_status_reg(), 0x8004);

        processor.flag_zero();
        assert_eq!(processor.processor_status_reg(), 0x8002);

        processor.flag_positive();
        assert_eq!(processor.processor_status_reg(), 0x8001);

        processor.set_priority(5);
        assert_eq!(processor.processor_status_reg(), 0x8501);
    }
}
