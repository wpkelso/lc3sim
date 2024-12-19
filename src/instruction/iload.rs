use crate::{
    defs::{LC3Word, RegAddr},
    executors::LC3,
    instruction::{
        args::{InstrOffset6, InstrPCOffset9},
        get_bits, get_opcode, set_condition_codes, Instruction,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ILoad {
    Std(InstrPCOffset9),      //LD
    Indirect(InstrPCOffset9), //LDI
    Reg(InstrOffset6),        //LDR
    Addr(InstrPCOffset9),     //LEA
}
pub const LD_OPCODE: u8 = 0b0010;
pub const LDI_OPCODE: u8 = 0b1010;
pub const LDR_OPCODE: u8 = 0b0110;
pub const LEA_OPCODE: u8 = 0b1110;
pub const ALL_LOAD_OPCODES: [u8; 4] = [LD_OPCODE, LDI_OPCODE, LDR_OPCODE, LEA_OPCODE];

impl Instruction for ILoad {
    fn execute<P: LC3>(self, processor: &mut P) {
        let result;
        match self {
            Self::Std(InstrPCOffset9 {
                target_reg,
                pc_offset,
            }) => {
                let target_addr: u16 = processor.pc() + 1 + pc_offset;
                result = processor.mem(target_addr);
                processor.set_reg(target_reg, result);
            }
            Self::Indirect(InstrPCOffset9 {
                target_reg,
                pc_offset,
            }) => {
                let target_addr: u16 = processor.pc() + 1 + pc_offset;
                let target_loc: u16 = processor.mem(target_addr);
                result = processor.mem(target_loc);
                processor.set_reg(target_reg, result);
            }
            Self::Reg(InstrOffset6 {
                target_reg,
                base_reg,
                offset,
            }) => {
                let target_addr = processor.reg(base_reg) + offset;
                result = processor.mem(target_addr);
                processor.set_reg(target_reg, result);
            }
            Self::Addr(InstrPCOffset9 {
                target_reg,
                pc_offset,
            }) => {
                result = processor.pc() + 1 + pc_offset;
                processor.set_reg(target_reg, result);
            }
        }

        set_condition_codes(processor, result);
    }

    fn parse(word: LC3Word) -> Option<Self>
    where
        Self: Sized,
    {
        let target_reg = RegAddr::panic_from_u16(get_bits(word, 11, 9));

        let opcode = get_opcode(word);
        match opcode {
            LD_OPCODE | LDI_OPCODE | LEA_OPCODE => {
                let pc_offset = get_bits(word, 8, 0);

                match opcode {
                    LD_OPCODE => Some(Self::Std(InstrPCOffset9 {
                        target_reg,
                        pc_offset,
                    })),
                    LDI_OPCODE => Some(Self::Indirect(InstrPCOffset9 {
                        target_reg,
                        pc_offset,
                    })),
                    LEA_OPCODE => Some(Self::Addr(InstrPCOffset9 {
                        target_reg,
                        pc_offset,
                    })),
                    x => unreachable!("{x} was not explicitly enumerated in the parent match!"),
                }
            }
            LDR_OPCODE => {
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
