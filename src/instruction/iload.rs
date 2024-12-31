use crate::{
    defs::{LC3MemAddr, LC3Word, RegAddr},
    executors::LC3,
    instruction::{
        args::{InstrOffset6, InstrPCOffset9},
        get_bits, get_opcode, set_condition_codes, Instruction, InstructionErr,
    },
    util::{apply_offset, shift_to_signed, shift_to_unsigned},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ILoad {
    Std(InstrPCOffset9),      //LD
    Indirect(InstrPCOffset9), //LDI
    Reg(InstrOffset6),        //LDR
    Addr(InstrPCOffset9),     //LEA
}
pub const LD_OPCODE: u8 = 0b0010;
pub const LDI_OPCODE: u8 = 0b1010;
pub const LDR_OPCODE: u8 = 0b0110;
pub const LEA_OPCODE: u8 = 0b1110;
pub const ALL_LOAD_OPCODES: [u8; 4] = [LD_OPCODE, LDI_OPCODE, LDR_OPCODE, LEA_OPCODE];

impl Instruction for ILoad {
    fn execute<P: LC3>(self, processor: &mut P) -> Result<(), InstructionErr> {
        let result;
        let advanced_addr: LC3MemAddr = processor.pc().wrapping_add(1);

        match self {
            Self::Std(InstrPCOffset9 {
                target_reg,
                pc_offset,
            }) => {
                let target_addr = apply_offset(advanced_addr, pc_offset);
                result = processor.mem(target_addr);
                processor.set_reg(target_reg, result);
            }
            Self::Indirect(InstrPCOffset9 {
                target_reg,
                pc_offset,
            }) => {
                let target_addr = apply_offset(advanced_addr, pc_offset);
                let target_loc: u16 = processor.mem(target_addr);
                result = processor.mem(target_loc);
                processor.set_reg(target_reg, result);
            }
            Self::Reg(InstrOffset6 {
                target_reg,
                base_reg,
                offset,
            }) => {
                let target_addr = apply_offset(processor.reg(base_reg), offset);
                result = processor.mem(target_addr);
                processor.set_reg(target_reg, result);
            }
            Self::Addr(InstrPCOffset9 {
                target_reg,
                pc_offset,
            }) => {
                result = apply_offset(advanced_addr, pc_offset);
                processor.set_reg(target_reg, result);
            }
        }

        set_condition_codes(processor, result);

        Ok(())
    }

    fn parse(word: LC3Word) -> Option<Self>
    where
        Self: Sized,
    {
        let target_reg = RegAddr::panic_from_u16(get_bits(word, 11, 9));

        let opcode = get_opcode(word);
        match opcode {
            LD_OPCODE | LDI_OPCODE | LEA_OPCODE => {
                // Shifting sign extends negative numbers
                let pc_offset = shift_to_signed::<{ LC3Word::BITS - 9 }>(get_bits(word, 8, 0));

                match opcode {
                    LD_OPCODE => Some(Self::Std(InstrPCOffset9 {
                        target_reg,
                        pc_offset,
                    })),
                    LDI_OPCODE => Some(Self::Indirect(InstrPCOffset9 {
                        target_reg,
                        pc_offset,
                    })),
                    LEA_OPCODE => Some(Self::Addr(InstrPCOffset9 {
                        target_reg,
                        pc_offset,
                    })),
                    x => unreachable!("{x} was not explicitly enumerated in the parent match!"),
                }
            }
            LDR_OPCODE => {
                let base_reg = RegAddr::panic_from_u16(get_bits(word, 8, 6));
                let offset = shift_to_signed::<{ LC3Word::BITS - 6 }>(get_bits(word, 5, 0));

                Some(Self::Reg(InstrOffset6 {
                    target_reg,
                    base_reg,
                    offset,
                }))
            }
            // Not in any of the valid opcodes
            _ => None,
        }
    }
}

impl From<ILoad> for LC3Word {
    fn from(value: ILoad) -> Self {
        const LD_BASE: LC3Word = (LD_OPCODE as LC3Word) << 12;
        const LDI_BASE: LC3Word = (LDI_OPCODE as LC3Word) << 12;
        const LDR_BASE: LC3Word = (LDR_OPCODE as LC3Word) << 12;
        const LEA_BASE: LC3Word = (LEA_OPCODE as LC3Word) << 12;

        match value {
            ILoad::Std(InstrPCOffset9 {
                target_reg,
                pc_offset,
            }) => {
                LD_BASE
                    | (LC3Word::from(target_reg) << 9)
                    | shift_to_unsigned::<{ LC3Word::BITS - 9 }>(pc_offset)
            }
            ILoad::Indirect(InstrPCOffset9 {
                target_reg,
                pc_offset,
            }) => {
                LDI_BASE
                    | (LC3Word::from(target_reg) << 9)
                    | shift_to_unsigned::<{ LC3Word::BITS - 9 }>(pc_offset)
            }
            ILoad::Reg(InstrOffset6 {
                target_reg,
                base_reg,
                offset,
            }) => {
                LDR_BASE
                    | (LC3Word::from(target_reg) << 9)
                    | (LC3Word::from(base_reg) << 6)
                    | shift_to_unsigned::<{ LC3Word::BITS - 6 }>(offset)
            }
            ILoad::Addr(InstrPCOffset9 {
                target_reg,
                pc_offset,
            }) => {
                LEA_BASE
                    | (LC3Word::from(target_reg) << 9)
                    | shift_to_unsigned::<{ LC3Word::BITS - 9 }>(pc_offset)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reject_invalid_opcodes() {
        // All other opcodes
        let invalid_opcodes =
            (0..LC3Word::MAX).filter(|word| !ALL_LOAD_OPCODES.contains(&((word >> 12) as u8)));

        for invalid in invalid_opcodes {
            assert!(ILoad::parse(invalid).is_none())
        }
    }

    mod ld {
        use super::*;

        const BASE_OPCODE: u16 = (LD_OPCODE as u16) << 12;
        const BITMASK_NINE: LC3Word = 1 << 9;

        #[test]
        fn parse() {
            for dr in 0..8 {
                let with_dr = BASE_OPCODE | (dr << 9);
                for imm in 0..BITMASK_NINE {
                    let full = with_dr | imm;

                    if let ILoad::Std(parsed) = ILoad::parse(full).unwrap() {
                        assert_eq!(parsed.target_reg as u16, dr);
                        assert_eq!(
                            parsed.pc_offset,
                            shift_to_signed::<{ LC3Word::BITS - 9 }>(imm)
                        );
                    } else {
                        panic!("Must parse as ld!")
                    }
                }
            }
        }
    }

    mod ldi {
        use super::*;

        const BASE_OPCODE: u16 = (LDI_OPCODE as u16) << 12;
        const BITMASK_NINE: LC3Word = 1 << 9;

        #[test]
        fn parse() {
            for dr in 0..8 {
                let with_dr = BASE_OPCODE | (dr << 9);
                for imm in 0..BITMASK_NINE {
                    let full = with_dr | imm;

                    if let ILoad::Indirect(parsed) = ILoad::parse(full).unwrap() {
                        assert_eq!(parsed.target_reg as u16, dr);
                        assert_eq!(
                            parsed.pc_offset,
                            shift_to_signed::<{ LC3Word::BITS - 9 }>(imm)
                        );
                    } else {
                        panic!("Must parse as ldi!")
                    }
                }
            }
        }
    }

    mod lea {
        use super::*;

        const BASE_OPCODE: u16 = (LEA_OPCODE as u16) << 12;
        const BITMASK_NINE: LC3Word = 1 << 9;

        #[test]
        fn parse() {
            for dr in 0..8 {
                let with_dr = BASE_OPCODE | (dr << 9);
                for imm in 0..BITMASK_NINE {
                    let full = with_dr | imm;

                    if let ILoad::Addr(parsed) = ILoad::parse(full).unwrap() {
                        assert_eq!(parsed.target_reg as u16, dr);
                        assert_eq!(
                            parsed.pc_offset,
                            shift_to_signed::<{ LC3Word::BITS - 9 }>(imm)
                        );
                    } else {
                        panic!("Must parse as lea!")
                    }
                }
            }
        }
    }

    mod ldr {
        use crate::defs::SignedLC3Word;

        use super::*;

        const BASE_OPCODE: u16 = (LDR_OPCODE as u16) << 12;

        #[test]
        fn parse() {
            for dr in 0..8 {
                let with_dr = BASE_OPCODE | (dr << 9);
                for sr in 0..8 {
                    let with_sr = with_dr | (sr << 6);
                    for imm in 0..0b11111 {
                        let full = with_sr | imm;

                        if let ILoad::Reg(parsed) = ILoad::parse(full).unwrap() {
                            assert_eq!(parsed.target_reg as u16, dr);
                            assert_eq!(parsed.base_reg as u16, sr);
                            assert_eq!(parsed.offset, imm as SignedLC3Word);
                        } else {
                            panic!("Must parse as ldr!")
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn reconstruct() {
        let valid_opcodes =
            (0..LC3Word::MAX).filter(|word| ALL_LOAD_OPCODES.contains(&((word >> 12) as u8)));

        for valid in valid_opcodes {
            assert_eq!(LC3Word::from(ILoad::parse(valid).unwrap()), valid)
        }
    }
}
