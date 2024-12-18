use crate::{
    defs::{LC3Word, RegAddr},
    executors::LC3,
    instruction::{args::InstrRegOnly, get_bits, get_opcode, set_condition_codes, Instruction},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct INot(pub InstrRegOnly);
const NOT_OPCODE: u8 = 0b1001;

impl Instruction for INot {
    fn execute<P: LC3>(self, processor: &mut P) {
        let InstrRegOnly { dest_reg, src_reg } = self.0;

        let dest;
        let result = {
            dest = dest_reg;
            !processor.reg(src_reg)
        };
        processor.set_reg(dest, result);
        set_condition_codes(processor, result);
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
