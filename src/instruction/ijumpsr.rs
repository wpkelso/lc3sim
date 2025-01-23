use crate::{
    defs::{LC3Word, RegAddr},
    executors::LC3,
    instruction::{
        args::InstrPCOffset11, get_bit, get_bits, get_opcode, Instruction, InstructionErr,
    },
    util::{apply_offset, shift_to_signed, shift_to_unsigned},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IJumpSubRoutine {
    Offset(InstrPCOffset11), //JSR
    Reg(RegAddr),            //JSRR treated as an offset6 with an offset of 0
}
pub const JSR_OPCODE: u8 = 0b0100;

impl Instruction for IJumpSubRoutine {
    fn execute<P: LC3>(self, processor: &mut P) -> Result<(), InstructionErr> {
        processor.set_reg(RegAddr::Seven, processor.pc()); //save return address
        let jump_addr = match self {
            Self::Offset(InstrPCOffset11 { pc_offset }) => {
                //JSR
                apply_offset(processor.pc(), pc_offset)
            }
            Self::Reg(base_reg) => {
                //JSRR
                processor.reg(base_reg)
            }
        };
        processor.set_pc(jump_addr);

        Ok(())
    }

    fn parse(word: LC3Word) -> Option<Self>
    where
        Self: Sized,
    {
        if get_opcode(word) == JSR_OPCODE {
            if get_bit(word, 11) == 1 {
                Some(Self::Offset(InstrPCOffset11 {
                    pc_offset: shift_to_signed::<{ LC3Word::BITS - 11 }>(get_bits(word, 10, 0)),
                }))
            } else if (get_bits(word, 11, 9) == 0) && (get_bits(word, 5, 0) == 0) {
                Some(Self::Reg(RegAddr::panic_from_u16(get_bits(word, 8, 6))))
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl From<IJumpSubRoutine> for LC3Word {
    fn from(value: IJumpSubRoutine) -> Self {
        const JSR_BASE: LC3Word = (JSR_OPCODE as LC3Word) << 12;

        match value {
            IJumpSubRoutine::Reg(reg) => JSR_BASE | (LC3Word::from(reg) << 6),
            IJumpSubRoutine::Offset(offset) => {
                JSR_BASE | (1 << 11) | shift_to_unsigned::<{ LC3Word::BITS - 11 }>(offset.pc_offset)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::instruction::TWELVE_SET;

    use super::*;

    const BASE_OPCODE: u16 = (JSR_OPCODE as u16) << 12;
    const BITMASK_11: u16 = 1 << 11;
    const BITMASK_11_9: u16 = 0b111 << 9;
    const BITMASK_5_0: u16 = (1 << 6) - 1;

    #[test]
    fn reject_invalid_opcodes() {
        // All other opcodes
        let invalid_opcodes =
            (0..LC3Word::MAX).filter(|word| (word >> 12) != (JSR_OPCODE as LC3Word));

        for invalid in invalid_opcodes {
            assert!(IJumpSubRoutine::parse(invalid).is_none())
        }
    }

    #[test]
    fn reject_invalid_parses() {
        let invalid_parses = (BASE_OPCODE..(BASE_OPCODE + TWELVE_SET)).filter(|word| {
            ((word & BITMASK_11) == 0) && (word & (BITMASK_11_9 | BITMASK_5_0) != 0)
        });

        for invalid in invalid_parses {
            assert!(IJumpSubRoutine::parse(invalid).is_none(),)
        }
    }

    #[test]
    fn parse_reg() {
        for dr in 0..8 {
            let full = BASE_OPCODE | (dr << 6);
            if let IJumpSubRoutine::Reg(parsed) = IJumpSubRoutine::parse(full).unwrap() {
                assert_eq!(parsed as u16, dr);
            } else {
                panic!("Must parse as register!")
            }
        }
    }

    #[test]
    fn parse_offset() {
        let base = BASE_OPCODE | (1 << 11);

        for offset in 0..(1 << 11) {
            let full = base | offset;
            if let IJumpSubRoutine::Offset(parsed) = IJumpSubRoutine::parse(full).unwrap() {
                assert_eq!(
                    parsed.pc_offset,
                    shift_to_signed::<{ LC3Word::BITS - 11 }>(offset)
                );
            } else {
                panic!("Must parse as register!")
            }
        }
    }

    #[test]
    fn reconstruct() {
        let valid_opcodes = (BASE_OPCODE..(BASE_OPCODE + TWELVE_SET)).filter(|word| {
            ((word & BITMASK_11) != 0) || (word | (BITMASK_11_9 & BITMASK_5_0) == 0)
        });

        for valid in valid_opcodes {
            assert_eq!(LC3Word::from(IJumpSubRoutine::parse(valid).unwrap()), valid)
        }
    }
}
