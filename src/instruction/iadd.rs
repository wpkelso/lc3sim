use crate::{
    defs::{LC3Word, RegAddr},
    executors::LC3,
    instruction::{
        args::{InstrRegImm, InstrRegReg},
        get_bits, get_opcode, set_condition_codes, Instruction,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IAdd {
    Reg(InstrRegReg),
    Imm(InstrRegImm),
}
pub const ADD_OPCODE: u8 = 0b0001;

impl Instruction for IAdd {
    fn execute<P: LC3>(self, processor: &mut P) {
        let dest;
        let result = match self {
            Self::Reg(InstrRegReg {
                dest_reg,
                src_reg_1,
                src_reg_2,
            }) => {
                dest = dest_reg;
                processor.reg(src_reg_1) + processor.reg(src_reg_2)
            }
            Self::Imm(InstrRegImm {
                dest_reg,
                src_reg,
                imm,
            }) => {
                dest = dest_reg;
                processor.reg(src_reg) + imm
            }
        };
        processor.set_reg(dest, result);
        set_condition_codes(processor, result);
    }

    fn parse(word: LC3Word) -> Option<Self>
    where
        Self: Sized,
    {
        if get_opcode(word) == ADD_OPCODE {
            // 3 bits is always a valid RegAddr
            let dest_reg = RegAddr::panic_from_u16(get_bits(word, 11, 9));
            // 3 bits is always a valid RegAddr
            let src_reg_1 = RegAddr::panic_from_u16(get_bits(word, 8, 6));

            if get_bits(word, 5, 5) == 0 {
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
