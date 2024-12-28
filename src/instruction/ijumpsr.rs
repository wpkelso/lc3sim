use crate::{
    defs::{LC3Word, RegAddr},
    executors::LC3,
    instruction::{
        args::InstrPCOffset11, get_bit, get_bits, get_opcode, Instruction, InstructionErr,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
                processor.pc() + pc_offset
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
                    pc_offset: get_bits(word, 10, 0),
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
