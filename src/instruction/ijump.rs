use crate::{
    defs::{LC3Word, RegAddr},
    executors::LC3,
    instruction::{
        args::{ConditionCodes, InstrRegImm, InstrRegOnly, InstrRegReg},
        get_bit, get_bits, get_opcode, set_condition_codes, Instruction,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IJump {
    Instr(RegAddr), //not strictly an offset6, but doesn't matter here
    Ret,            //RET and RETI are included here, as they are functionally special cases of JMP
    InterRet,
}
const JMP_OPCODE: u8 = 0b1100;
const RTI_OPCODE: u8 = 0b1000;
const ALL_JUMP_OPCODES: [u8; 2] = [JMP_OPCODE, RTI_OPCODE];

impl Instruction for IJump {
    fn execute<P: LC3>(self, processor: &mut P) {
        let dest = match self {
            Self::Instr(base_reg) => base_reg,
            Self::Ret => RegAddr::Seven,
            Self::InterRet => {
                unimplemented!()
            }
        };
        processor.set_pc(processor.reg(dest));
    }

    fn parse(word: LC3Word) -> Option<Self>
    where
        Self: Sized,
    {
        match get_opcode(word) {
            JMP_OPCODE => {
                if (get_bits(word, 11, 9) == 0) && (get_bits(word, 5, 0) == 0) {
                    let dest = RegAddr::panic_from_u16(get_bits(word, 8, 6));

                    if dest == RegAddr::Seven {
                        Some(Self::Ret)
                    } else {
                        Some(Self::Instr(dest))
                    }
                } else {
                    None
                }
            }
            RTI_OPCODE => {
                if get_bits(word, 11, 0) == 0 {
                    Some(Self::InterRet)
                } else {
                    None
                }
            }
            // Not one of two valid opcodes
            _ => None,
        }
    }
}
