use crate::{
    defs::{LC3Word, RegAddr},
    executors::LC3,
    instruction::{
        args::{InstrOffset6, InstrPCOffset9},
        get_bits, get_opcode, Instruction,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IStore {
    Std(InstrPCOffset9),      //ST
    Indirect(InstrPCOffset9), //STI
    Reg(InstrOffset6),        //STR
}
const ST_OPCODE: u8 = 0b0011;
const STI_OPCODE: u8 = 0b1011;
const STR_OPCODE: u8 = 0b0111;
const ALL_STORE_OPCODES: [u8; 3] = [ST_OPCODE, STI_OPCODE, STR_OPCODE];

impl Instruction for IStore {
    fn execute<P: LC3>(self, processor: &mut P) {
        match self {
            Self::Std(InstrPCOffset9 {
                target_reg,
                pc_offset,
            }) => {
                let target_addr: u16 = processor.pc() + 1 + pc_offset;
                processor.set_mem(target_addr, processor.reg(target_reg));
            }
            Self::Indirect(InstrPCOffset9 {
                target_reg,
                pc_offset,
            }) => {
                let calc_addr: u16 = processor.pc() + 1 + pc_offset;
                let target_addr: u16 = processor.mem(calc_addr);
                processor.set_mem(target_addr, processor.reg(target_reg));
            }
            Self::Reg(InstrOffset6 {
                target_reg,
                base_reg,
                offset,
            }) => {
                let target_addr: u16 = processor.reg(base_reg) + offset;
                processor.set_mem(target_addr, processor.reg(target_reg));
            }
        }
    }

    fn parse(word: LC3Word) -> Option<Self>
    where
        Self: Sized,
    {
        let target_reg = RegAddr::panic_from_u16(get_bits(word, 11, 9));

        let opcode = get_opcode(word);
        match opcode {
            ST_OPCODE | STI_OPCODE => {
                let pc_offset = get_bits(word, 8, 0);

                match opcode {
                    ST_OPCODE => Some(Self::Std(InstrPCOffset9 {
                        target_reg,
                        pc_offset,
                    })),
                    STI_OPCODE => Some(Self::Indirect(InstrPCOffset9 {
                        target_reg,
                        pc_offset,
                    })),
                    x => unreachable!("{x} was not explicitly enumerated in the parent match!"),
                }
            }
            STR_OPCODE => {
                let base_reg = RegAddr::panic_from_u16(get_bits(word, 8, 6));
                let offset = get_bits(word, 5, 0);

                Some(Self::Reg(InstrOffset6 {
                    target_reg,
                    base_reg,
                    offset,
                }))
            }
            // Not in any of the valid opcodes
            _ => None,
        }
    }
}
