use crate::{
    defs::{LC3Word, RegAddr},
    executors::LC3,
    instruction::{get_bits, get_opcode, Instruction, InstructionErr},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Trap {
    Getc,  // 0x20
    Out,   // 0x21
    PutS,  // 0x22
    In,    // 0x23
    PutSp, // 0x24
    Halt,  // 0x25
}
pub const TRAP_OPCODE: u8 = 0b1111;

const GETC: u16 = 0x20;
const OUT: u16 = 0x21;
const PUTS: u16 = 0x22;
const IN: u16 = 0x23;
const PUTSP: u16 = 0x24;
const HALT: u16 = 0x25;

impl Instruction for Trap {
    fn execute<P: LC3>(self, processor: &mut P) -> Result<(), InstructionErr> {
        let vector = match self {
            Trap::Getc => GETC,
            Trap::Out => OUT,
            Trap::PutS => PUTS,
            Trap::In => IN,
            Trap::PutSp => PUTSP,
            Trap::Halt => {
                processor.halt();
                HALT
            }
        };

        processor.set_reg(RegAddr::Seven, processor.pc() + 1);
        processor.set_pc(processor.mem(vector));

        Ok(())
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
                PUTSP => Some(Self::PutSp),
                HALT => Some(Self::Halt),
                _ => None,
            }
        } else {
            None
        }
    }
}

impl From<Trap> for LC3Word {
    fn from(value: Trap) -> Self {
        const BASE: LC3Word = (TRAP_OPCODE as LC3Word) << 12;

        match value {
            Trap::Getc => BASE | GETC,
            Trap::Out => BASE | OUT,
            Trap::PutS => BASE | PUTS,
            Trap::In => BASE | IN,
            Trap::PutSp => BASE | PUTSP,
            Trap::Halt => BASE | HALT,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE_OPCODE: u16 = (TRAP_OPCODE as u16) << 12;

    const ALL_VECS: [u16; 6] = [GETC, OUT, PUTS, IN, PUTSP, HALT];

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

    #[test]
    fn reconstruct() {
        let valid_opcodes = ALL_VECS.into_iter().map(|vec| BASE_OPCODE | vec);

        for valid in valid_opcodes {
            assert_eq!(LC3Word::from(Trap::parse(valid).unwrap()), valid)
        }
    }
}
