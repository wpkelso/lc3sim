use crate::{
    defs::LC3Word,
    executors::LC3,
    instruction::{
        args::ConditionCodes, get_bit, get_bits, get_opcode, Instruction, InstructionErr,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct IBranch {
    //while br roughly follows the bit assignment of PCoffset9,
    //this is treated as a special case for ease of implementation
    pub cond_codes: ConditionCodes,
    pub pc_offset: u16,
}
pub const BRANCH_OPCODE: u8 = 0b0000;

impl Instruction for IBranch {
    fn execute<P: LC3>(self, processor: &mut P) -> Result<(), InstructionErr> {
        let pos_condition = self.cond_codes.positive && processor.positive_cond();
        let zero_condition = self.cond_codes.zero && processor.zero_cond();
        let neg_condition = self.cond_codes.negative && processor.negative_cond();

        if pos_condition || zero_condition || neg_condition {
            processor.set_pc(processor.pc() + self.pc_offset);
        }

        Ok(())
    }

    fn parse(word: LC3Word) -> Option<Self>
    where
        Self: Sized,
    {
        if get_opcode(word) == BRANCH_OPCODE {
            let cond_codes = ConditionCodes {
                positive: get_bit(word, 9) == 1,
                negative: get_bit(word, 11) == 1,
                zero: get_bit(word, 10) == 1,
            };

            let pc_offset = get_bits(word, 8, 0);

            Some(Self {
                cond_codes,
                pc_offset,
            })
        } else {
            None
        }
    }
}

impl From<IBranch> for LC3Word {
    fn from(value: IBranch) -> Self {
        const BASE: LC3Word = (BRANCH_OPCODE as LC3Word) << 12;

        let mut with_cond = BASE;

        if value.cond_codes.negative {
            with_cond |= 1 << 11;
        }
        if value.cond_codes.zero {
            with_cond |= 1 << 10;
        }
        if value.cond_codes.positive {
            with_cond |= 1 << 9;
        }

        with_cond | value.pc_offset
    }
}

#[cfg(test)]
mod tests {
    use crate::instruction::TWELVE_SET;

    use super::*;

    const BITMASK_9: u16 = 1 << 9;
    const BASE_OPCODE: u16 = (BRANCH_OPCODE as u16) << 12;

    #[test]
    fn reject_invalid_opcodes() {
        // All other opcodes
        let invalid_opcodes = (0..LC3Word::MAX).filter(|word| (word >> 12) != BRANCH_OPCODE as u16);

        for invalid in invalid_opcodes {
            assert!(IBranch::parse(invalid).is_none())
        }
    }

    #[test]
    fn parse() {
        for offset in 0..BITMASK_9 {
            let with_offset = BASE_OPCODE | offset;

            for neg in [true, false] {
                for zero in [true, false] {
                    for pos in [true, false] {
                        let mut full = with_offset;
                        if neg {
                            full |= 1 << 11
                        }
                        if zero {
                            full |= 1 << 10
                        }
                        if pos {
                            full |= 1 << 9
                        }

                        let IBranch {
                            cond_codes,
                            pc_offset,
                        } = IBranch::parse(full).unwrap();

                        assert_eq!(pc_offset, offset);
                        assert_eq!(cond_codes.negative, neg);
                        assert_eq!(cond_codes.zero, zero);
                        assert_eq!(cond_codes.positive, pos);
                    }
                }
            }
        }
    }

    #[test]
    fn reconstruct() {
        let valid_opcodes = BASE_OPCODE..(BASE_OPCODE + TWELVE_SET);

        for valid in valid_opcodes {
            assert_eq!(LC3Word::from(IBranch::parse(valid).unwrap()), valid)
        }
    }
}
