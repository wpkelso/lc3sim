use crate::{
    defs::{LC3Word, RegAddr},
    executors::LC3,
    instruction::{
        args::{InstrOffset6, InstrPCOffset9},
        get_bits, get_opcode, Instruction, InstructionErr,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IStore {
    Std(InstrPCOffset9),      //ST
    Indirect(InstrPCOffset9), //STI
    Reg(InstrOffset6),        //STR
}
pub const ST_OPCODE: u8 = 0b0011;
pub const STI_OPCODE: u8 = 0b1011;
pub const STR_OPCODE: u8 = 0b0111;
pub const ALL_STORE_OPCODES: [u8; 3] = [ST_OPCODE, STI_OPCODE, STR_OPCODE];

impl Instruction for IStore {
    fn execute<P: LC3>(self, processor: &mut P) -> Result<(), InstructionErr> {
        match self {
            Self::Std(InstrPCOffset9 {
                target_reg,
                pc_offset,
            }) => {
                let target_addr: u16 = processor.pc() + 1 + pc_offset;
                processor.set_mem(target_addr, processor.reg(target_reg));
            }
            Self::Indirect(InstrPCOffset9 {
                target_reg,
                pc_offset,
            }) => {
                let calc_addr: u16 = processor.pc() + 1 + pc_offset;
                let target_addr: u16 = processor.mem(calc_addr);
                processor.set_mem(target_addr, processor.reg(target_reg));
            }
            Self::Reg(InstrOffset6 {
                target_reg,
                base_reg,
                offset,
            }) => {
                let target_addr: u16 = processor.reg(base_reg) + offset;
                processor.set_mem(target_addr, processor.reg(target_reg));
            }
        }

        Ok(())
    }

    fn parse(word: LC3Word) -> Option<Self>
    where
        Self: Sized,
    {
        let target_reg = RegAddr::panic_from_u16(get_bits(word, 11, 9));

        let opcode = get_opcode(word);
        match opcode {
            ST_OPCODE | STI_OPCODE => {
                let pc_offset = get_bits(word, 8, 0);

                match opcode {
                    ST_OPCODE => Some(Self::Std(InstrPCOffset9 {
                        target_reg,
                        pc_offset,
                    })),
                    STI_OPCODE => Some(Self::Indirect(InstrPCOffset9 {
                        target_reg,
                        pc_offset,
                    })),
                    x => unreachable!("{x} was not explicitly enumerated in the parent match!"),
                }
            }
            STR_OPCODE => {
                let base_reg = RegAddr::panic_from_u16(get_bits(word, 8, 6));
                let offset = get_bits(word, 5, 0);

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

impl From<IStore> for LC3Word {
    fn from(value: IStore) -> Self {
        const ST_BASE: LC3Word = (ST_OPCODE as LC3Word) << 12;
        const STI_BASE: LC3Word = (STI_OPCODE as LC3Word) << 12;
        const STR_BASE: LC3Word = (STR_OPCODE as LC3Word) << 12;

        match value {
            IStore::Std(InstrPCOffset9 {
                target_reg,
                pc_offset,
            }) => ST_BASE | (LC3Word::from(target_reg) << 9) | pc_offset,
            IStore::Indirect(InstrPCOffset9 {
                target_reg,
                pc_offset,
            }) => STI_BASE | (LC3Word::from(target_reg) << 9) | pc_offset,
            IStore::Reg(InstrOffset6 {
                target_reg,
                base_reg,
                offset,
            }) => {
                STR_BASE
                    | (LC3Word::from(target_reg) << 9)
                    | (LC3Word::from(base_reg) << 6)
                    | offset
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
            (0..LC3Word::MAX).filter(|word| !ALL_STORE_OPCODES.contains(&((word >> 12) as u8)));

        for invalid in invalid_opcodes {
            assert!(IStore::parse(invalid).is_none())
        }
    }

    mod st {
        use super::*;

        const BASE_OPCODE: u16 = (ST_OPCODE as u16) << 12;
        const BITMASK_NINE: LC3Word = 1 << 9;

        #[test]
        fn parse() {
            for dr in 0..8 {
                let with_dr = BASE_OPCODE | (dr << 9);
                for imm in 0..BITMASK_NINE {
                    let full = with_dr | imm;

                    if let IStore::Std(parsed) = IStore::parse(full).unwrap() {
                        assert_eq!(parsed.target_reg as u16, dr);
                        assert_eq!(parsed.pc_offset, imm);
                    } else {
                        panic!("Must parse as st!")
                    }
                }
            }
        }
    }

    mod sti {
        use super::*;

        const BASE_OPCODE: u16 = (STI_OPCODE as u16) << 12;
        const BITMASK_NINE: LC3Word = 1 << 9;

        #[test]
        fn parse() {
            for dr in 0..8 {
                let with_dr = BASE_OPCODE | (dr << 9);
                for imm in 0..BITMASK_NINE {
                    let full = with_dr | imm;

                    if let IStore::Indirect(parsed) = IStore::parse(full).unwrap() {
                        assert_eq!(parsed.target_reg as u16, dr);
                        assert_eq!(parsed.pc_offset, imm);
                    } else {
                        panic!("Must parse as sti!")
                    }
                }
            }
        }
    }

    mod str {
        use super::*;

        const BASE_OPCODE: u16 = (STR_OPCODE as u16) << 12;

        #[test]
        fn parse() {
            for dr in 0..8 {
                let with_dr = BASE_OPCODE | (dr << 9);
                for sr in 0..8 {
                    let with_sr = with_dr | (sr << 6);
                    for imm in 0..0b11111 {
                        let full = with_sr | imm;

                        if let IStore::Reg(parsed) = IStore::parse(full).unwrap() {
                            assert_eq!(parsed.target_reg as u16, dr);
                            assert_eq!(parsed.base_reg as u16, sr);
                            assert_eq!(parsed.offset, imm);
                        } else {
                            panic!("Must parse as str!")
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn reconstruct() {
        let valid_opcodes =
            (0..LC3Word::MAX).filter(|word| ALL_STORE_OPCODES.contains(&((word >> 12) as u8)));

        for valid in valid_opcodes {
            assert_eq!(LC3Word::from(IStore::parse(valid).unwrap()), valid)
        }
    }
}
