use crate::{
    defs::{LC3Word, RegAddr},
    executors::LC3,
    instruction::{
        args::InstrRegOnly, get_bits, get_opcode, set_condition_codes, Instruction, InstructionErr,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct INot(pub InstrRegOnly);
pub const NOT_OPCODE: u8 = 0b1001;

impl Instruction for INot {
    fn execute<P: LC3>(self, processor: &mut P) -> Result<(), InstructionErr> {
        let InstrRegOnly { dest_reg, src_reg } = self.0;

        let dest;
        let result = {
            dest = dest_reg;
            !processor.reg(src_reg)
        };
        processor.set_reg(dest, result);
        set_condition_codes(processor, result);

        Ok(())
    }

    fn parse(word: LC3Word) -> Option<Self>
    where
        Self: Sized,
    {
        const TRAILING: u16 = 0b11111;

        if (get_opcode(word) == NOT_OPCODE) && (get_bits(word, 5, 0) == TRAILING) {
            // 3 bits is always a valid RegAddr
            let dest_reg = RegAddr::panic_from_u16(get_bits(word, 11, 9));
            // 3 bits is always a valid RegAddr
            let src_reg = RegAddr::panic_from_u16(get_bits(word, 8, 6));

            Some(Self(InstrRegOnly { dest_reg, src_reg }))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::instruction::TWELVE_SET;

    use super::*;

    const BOTTOM_FIVE: u16 = (1 << 4) | (1 << 3) | (1 << 2) | (1 << 1) | 1;
    const BASE_OPCODE: u16 = (NOT_OPCODE as u16) << 12;

    #[test]
    fn reject_invalid_parses() {
        // Invalid when bottom 5 bits aren't set
        let invalid_parses = (BASE_OPCODE..(BASE_OPCODE + TWELVE_SET))
            .filter(|word| (word & BOTTOM_FIVE) != BOTTOM_FIVE);

        for invalid in invalid_parses {
            assert!(INot::parse(invalid).is_none())
        }
    }

    #[test]
    fn reject_invalid_opcodes() {
        // All other opcodes
        let invalid_opcodes = (0..LC3Word::MAX).filter(|word| (word >> 12) != NOT_OPCODE as u16);

        for invalid in invalid_opcodes {
            assert!(INot::parse(invalid).is_none())
        }
    }

    #[test]
    fn parse() {
        // Without immediate flag
        let base = BASE_OPCODE;

        for dr in 0..7 {
            let with_dr = base | (dr << 9);
            for sr in 0..7 {
                let with_sr = with_dr | (sr << 6);
                let full = with_sr | BOTTOM_FIVE;

                let parsed = INot::parse(full).unwrap().0;
                assert_eq!(parsed.dest_reg as u16, dr);
                assert_eq!(parsed.src_reg as u16, sr);
            }
        }
    }
}
