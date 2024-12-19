use crate::{
    defs::LC3Word,
    executors::LC3,
    instruction::{get_bits, get_opcode, Instruction},
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

impl Instruction for Trap {
    fn execute<P: LC3>(self, processor: &mut P) {
        unimplemented!()
    }

    fn parse(word: LC3Word) -> Option<Self>
    where
        Self: Sized,
    {
        const GETC: u16 = 0x20;
        const OUT: u16 = 0x21;
        const PUTS: u16 = 0x22;
        const IN: u16 = 0x23;
        const HALT: u16 = 0x24;

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
