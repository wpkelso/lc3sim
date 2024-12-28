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
