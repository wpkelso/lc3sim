use crate::{
    defs::{LC3Word, RegAddr, STACK_REG},
    executors::LC3,
    instruction::{get_bits, get_opcode, Instruction, InstructionErr},
};

use super::InsufficientPerms;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IJump {
    Instr(RegAddr),
    PrivClear(RegAddr), // Clears privilege bit
    Ret, //RET and RETI are included here, as they are functionally special cases of JMP
    InterRet,
}
pub const JMP_OPCODE: u8 = 0b1100;
pub const RTI_OPCODE: u8 = 0b1000;
pub const ALL_JUMP_OPCODES: [u8; 2] = [JMP_OPCODE, RTI_OPCODE];

impl Instruction for IJump {
    fn execute<P: LC3>(self, processor: &mut P) -> Result<(), InstructionErr> {
        let dest = match self {
            Self::Instr(base_reg) => processor.reg(base_reg),
            Self::PrivClear(base_reg) => {
                processor.set_privileged(false);
                processor.reg(base_reg)
            }
            Self::Ret => processor.reg(RegAddr::Seven),
            Self::InterRet => {
                if !processor.privileged() {
                    return Err(InsufficientPerms.into());
                } else {
                    let stack_reg = processor.reg(STACK_REG);

                    // Pop PC and PSR from the supervisor stack
                    let pc = processor.mem(stack_reg + 1);
                    let psr = processor.mem(stack_reg + 2);
                    processor.set_reg(STACK_REG, stack_reg - 2);

                    // Restoring the status register also assigns STACK_REG
                    // correctly
                    processor.set_processor_status_reg(psr);

                    pc
                }
            }
        };
        processor.set_pc(dest);
        Ok(())
    }

    fn parse(word: LC3Word) -> Option<Self>
    where
        Self: Sized,
    {
        match get_opcode(word) {
            JMP_OPCODE => {
                if (get_bits(word, 11, 9) == 0) && (get_bits(word, 5, 1) == 0) {
                    let dest = RegAddr::panic_from_u16(get_bits(word, 8, 6));

                    if get_bits(word, 0, 0) == 1 {
                        Some(Self::PrivClear(dest))
                    } else if dest == RegAddr::Seven {
                        Some(Self::Ret)
                    } else {
                        Some(Self::Instr(dest))
                    }
                } else {
                    None
                }
            }
            RTI_OPCODE => {
                if get_bits(word, 11, 0) == 0 {
                    Some(Self::InterRet)
                } else {
                    None
                }
            }
            // Not one of two valid opcodes
            _ => None,
        }
    }
}

impl From<IJump> for LC3Word {
    fn from(value: IJump) -> Self {
        const JMP_BASE: LC3Word = (JMP_OPCODE as LC3Word) << 12;
        const RET_FULL: LC3Word = JMP_BASE | (0b111 << 6);
        const RTI_FULL: LC3Word = (RTI_OPCODE as LC3Word) << 12;

        match value {
            IJump::Instr(base_r) => JMP_BASE | (LC3Word::from(base_r) << 6),
            IJump::PrivClear(base_r) => JMP_BASE | (LC3Word::from(base_r) << 6) | 1,
            IJump::Ret => RET_FULL,
            IJump::InterRet => RTI_FULL,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::instruction::TWELVE_SET;

    use super::*;

    #[test]
    fn reject_invalid_opcodes() {
        // All other opcodes
        let invalid_opcodes =
            (0..LC3Word::MAX).filter(|word| !ALL_JUMP_OPCODES.contains(&((word >> 12) as u8)));

        for invalid in invalid_opcodes {
            assert!(IJump::parse(invalid).is_none())
        }
    }

    mod jmp {
        use super::*;

        const BASE_OPCODE: u16 = (JMP_OPCODE as u16) << 12;

        const BITMASK_11_9: u16 = 0b111 << 9;
        const BITMASK_5_1: u16 = ((1 << 5) - 1) << 1;

        #[test]
        fn reject_invalid_parses() {
            let invalid_parses = (BASE_OPCODE..(BASE_OPCODE + TWELVE_SET))
                .filter(|word| (word & (BITMASK_11_9 | BITMASK_5_1)) != 0);

            for invalid in invalid_parses {
                assert!(IJump::parse(invalid).is_none())
            }
        }

        #[test]
        fn parse_reg() {
            for dr in 0..7 {
                let full = BASE_OPCODE | (dr << 6);
                if let IJump::Instr(parsed) = IJump::parse(full).unwrap() {
                    assert_eq!(parsed as u16, dr);
                } else {
                    panic!("Must parse as register!")
                }
            }

            let dr = 7;
            let full = BASE_OPCODE | (dr << 6);
            assert_eq!(IJump::parse(full).unwrap(), IJump::Ret);
        }

        #[test]
        fn parse_special() {
            let base = BASE_OPCODE | 1;

            for dr in 0..7 {
                let full = base | (dr << 6);
                if let IJump::PrivClear(parsed) = IJump::parse(full).unwrap() {
                    assert_eq!(parsed as u16, dr);
                } else {
                    panic!("Must parse as privilege clear!")
                }
            }

            let dr = 7;
            let full = BASE_OPCODE | (dr << 6);
            assert_eq!(IJump::parse(full).unwrap(), IJump::Ret);
        }

        #[test]
        fn reconstruct() {
            let valid_opcodes = (BASE_OPCODE..(BASE_OPCODE + TWELVE_SET))
                .filter(|word| (word & (BITMASK_11_9 | BITMASK_5_1)) == 0);

            for valid in valid_opcodes {
                assert_eq!(LC3Word::from(IJump::parse(valid).unwrap()), valid)
            }
        }
    }

    mod rti {
        use crate::{
            defs::{IO_PRIORITY, KEYBOARD_INTERRUPT, OS_SUPER_STACK, SUPERVISOR_SP_INIT},
            executors::{core::CoreLC3, StepFailure},
        };

        use super::*;

        const BASE_OPCODE: u16 = (RTI_OPCODE as u16) << 12;

        const BITMASK_11_0: u16 = (1 << 12) - 1;

        #[test]
        fn reject_invalid_parses() {
            let invalid_parses =
                (BASE_OPCODE..(BASE_OPCODE + TWELVE_SET)).filter(|word| (word & BITMASK_11_0) != 0);

            for invalid in invalid_parses {
                assert!(IJump::parse(invalid).is_none())
            }
        }

        #[test]
        fn full_jump() {
            const INIT_STACK_REG: LC3Word = 63;
            const { assert!(INIT_STACK_REG != (SUPERVISOR_SP_INIT - 2)) };

            let mut processor = CoreLC3::new();
            processor.set_privileged(false);
            processor.set_reg(STACK_REG, INIT_STACK_REG);

            // Execute the original interrupt
            processor.interrupt(KEYBOARD_INTERRUPT, Some(IO_PRIORITY));
            assert!(processor.privileged());
            assert_eq!(processor.pc(), KEYBOARD_INTERRUPT + 0x0100);
            assert_eq!(processor.reg(STACK_REG), SUPERVISOR_SP_INIT - 2);

            // Execute the return jump
            processor.set_mem(processor.pc(), IJump::InterRet.into());
            processor.step().unwrap();

            assert_eq!(processor.pc(), OS_SUPER_STACK);
            assert_eq!(processor.reg(STACK_REG), INIT_STACK_REG);
            assert!(!processor.privileged());
        }

        #[test]
        fn invalid_return() {
            let mut processor = CoreLC3::new();
            processor.set_privileged(false);

            // Fail this instruction due to insufficient perms
            processor.set_mem(processor.pc(), IJump::InterRet.into());
            assert!(matches!(
                processor.step(),
                Err(StepFailure::InsufficientPerms(_))
            ));
        }

        #[test]
        fn reconstruct() {
            assert_eq!(
                LC3Word::from(IJump::parse(BASE_OPCODE).unwrap()),
                BASE_OPCODE
            )
        }
    }
}
