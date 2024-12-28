use crate::{
    defs::{LC3Word, RegAddr},
    executors::LC3,
    instruction::{
        args::{InstrRegImm, InstrRegReg},
        get_bit, get_bits, get_opcode, set_condition_codes, Instruction, InstructionErr,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IAnd {
    Reg(InstrRegReg),
    Imm(InstrRegImm),
}
pub const AND_OPCODE: u8 = 0b0101;

impl Instruction for IAnd {
    fn execute<P: LC3>(self, processor: &mut P) -> Result<(), InstructionErr> {
        let dest;
        let result = match self {
            Self::Reg(InstrRegReg {
                dest_reg,
                src_reg_1,
                src_reg_2,
            }) => {
                dest = dest_reg;
                processor.reg(src_reg_1) & processor.reg(src_reg_2)
            }
            Self::Imm(InstrRegImm {
                dest_reg,
                src_reg,
                imm,
            }) => {
                dest = dest_reg;
                processor.reg(src_reg) & imm
            }
        };
        processor.set_reg(dest, result);
        set_condition_codes(processor, result);
        Ok(())
    }

    fn parse(word: LC3Word) -> Option<Self>
    where
        Self: Sized,
    {
        if get_opcode(word) == AND_OPCODE {
            // 3 bits is always a valid RegAddr
            let dest_reg = RegAddr::panic_from_u16(get_bits(word, 11, 9));
            // 3 bits is always a valid RegAddr
            let src_reg_1 = RegAddr::panic_from_u16(get_bits(word, 8, 6));

            if get_bit(word, 5) == 0 {
                Some(Self::Reg(InstrRegReg {
                    dest_reg,
                    src_reg_1,
                    // 3 bits is always a valid RegAddr
                    src_reg_2: RegAddr::panic_from_u16(get_bits(word, 2, 0)),
                }))
            } else {
                Some(Self::Imm(InstrRegImm {
                    dest_reg,
                    src_reg: src_reg_1,
                    imm: get_bits(word, 4, 0),
                }))
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::instruction::TWELVE_SET;

    use super::*;

    const BITMASK_5: u16 = 1 << 5;
    const BITMASK_4: u16 = 1 << 5;
    const BITMASK_3: u16 = 1 << 5;
    const BITMASK_4_3: u16 = BITMASK_4 | BITMASK_3;
    const BASE_OPCODE: u16 = (AND_OPCODE as u16) << 12;

    #[test]
    fn reject_invalid_parses() {
        // Only invalid state (with right opcode): bit 5 is unset but bits
        // 4 or 3 are set.
        let invalid_parses = (BASE_OPCODE..(BASE_OPCODE + TWELVE_SET))
            .filter(|word| (word & BITMASK_5) == 0)
            .filter(|word| (word & BITMASK_4_3) != 0);

        for invalid in invalid_parses {
            assert!(IAnd::parse(invalid).is_none())
        }
    }

    #[test]
    fn reject_invalid_opcodes() {
        // All other opcodes
        let invalid_opcodes = (0..LC3Word::MAX).filter(|word| (word >> 12) != AND_OPCODE as u16);

        for invalid in invalid_opcodes {
            assert!(IAnd::parse(invalid).is_none())
        }
    }

    #[test]
    fn parse_immediates() {
        // With immediate flag
        let base = BASE_OPCODE | BITMASK_5;

        for dr in 0..7 {
            let with_dr = base | (dr << 9);
            for sr in 0..7 {
                let with_sr = with_dr | (sr << 6);
                for imm in 0..0b11111 {
                    let full = with_sr | imm;

                    if let IAnd::Imm(parsed) = IAnd::parse(full).unwrap() {
                        assert_eq!(parsed.dest_reg as u16, dr);
                        assert_eq!(parsed.src_reg as u16, sr);
                        assert_eq!(parsed.imm, imm);
                    } else {
                        panic!("Must parse as immediate!")
                    }
                }
            }
        }
    }

    #[test]
    fn parse_reg() {
        // Without immediate flag
        let base = BASE_OPCODE;

        for dr in 0..7 {
            let with_dr = base | (dr << 9);
            for sr1 in 0..7 {
                let with_sr = with_dr | (sr1 << 6);
                for sr2 in 0..7 {
                    let full = with_sr | sr2;

                    if let IAnd::Reg(parsed) = IAnd::parse(full).unwrap() {
                        assert_eq!(parsed.dest_reg as u16, dr);
                        assert_eq!(parsed.src_reg_1 as u16, sr1);
                        assert_eq!(parsed.src_reg_2 as u16, sr2);
                    } else {
                        panic!("Must parse as register!")
                    }
                }
            }
        }
    }
}
