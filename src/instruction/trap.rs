use crate::{
    defs::LC3Word,
    executors::LC3,
    instruction::{get_bits, get_opcode, Instruction, InstructionErr},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Trap {
    Getc, // 0x20
    Out,  // 0x21
    PutS, // 0x22
    In,   // 0x23
    Halt, // 0x24
}
pub const TRAP_OPCODE: u8 = 0b1111;

const GETC: u16 = 0x20;
const OUT: u16 = 0x21;
const PUTS: u16 = 0x22;
const IN: u16 = 0x23;
const HALT: u16 = 0x24;

impl Instruction for Trap {
    fn execute<P: LC3>(self, processor: &mut P) -> Result<(), InstructionErr> {
        unimplemented!()
    }

    fn parse(word: LC3Word) -> Option<Self>
    where
        Self: Sized,
    {
        if (get_opcode(word) == TRAP_OPCODE) && (get_bits(word, 11, 8) == 0) {
            match get_bits(word, 7, 0) {
                GETC => Some(Self::Getc),
                OUT => Some(Self::Out),
                PUTS => Some(Self::PutS),
                IN => Some(Self::In),
                HALT => Some(Self::Halt),
                _ => None,
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE_OPCODE: u16 = (TRAP_OPCODE as u16) << 12;

    const ALL_VECS: [u16; 5] = [GETC, OUT, PUTS, IN, HALT];

    const FULL_11: u16 = (1 << 12) - 1;
    const BITMASK_11_8: u16 = (FULL_11 >> 8) << 8;
    const BITMASK_7_0: u16 = (1 << 8) - 1;

    #[test]
    fn reject_invalid_opcodes() {
        // All other opcodes
        let invalid_opcodes = (0..LC3Word::MAX).filter(|word| (word >> 12) != (TRAP_OPCODE as u16));

        for invalid in invalid_opcodes {
            assert!(Trap::parse(invalid).is_none())
        }
    }

    #[test]
    fn reject_invalid_parses() {
        // Invalid when bottom 5 bits aren't set
        let invalid_parses = (BASE_OPCODE..=LC3Word::MAX).filter(|word| {
            ((word & BITMASK_11_8) != 0) || !ALL_VECS.contains(&(word & BITMASK_7_0))
        });

        for invalid in invalid_parses {
            assert!(Trap::parse(invalid).is_none())
        }
    }

    #[test]
    fn parse() {
        assert_eq!(Trap::parse(BASE_OPCODE | GETC).unwrap(), Trap::Getc);
        assert_eq!(Trap::parse(BASE_OPCODE | OUT).unwrap(), Trap::Out);
        assert_eq!(Trap::parse(BASE_OPCODE | PUTS).unwrap(), Trap::PutS);
        assert_eq!(Trap::parse(BASE_OPCODE | IN).unwrap(), Trap::In);
        assert_eq!(Trap::parse(BASE_OPCODE | HALT).unwrap(), Trap::Halt);
    }
}
