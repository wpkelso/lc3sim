//TODO: TRAP instructions

use crate::{
    defs::{LC3Word, RegAddr, SignedLC3Word},
    executors::LC3,
};

pub trait Instruction {
    /// Run this instruction on `P`, producing all outputs and side effects.
    fn execute<P: LC3>(self, processor: &mut P);

    /// Convert the word into this instruction, if possible.
    fn parse(word: LC3Word) -> Option<Self>
    where
        Self: Sized;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct InstrRegImm {
    pub dest_reg: RegAddr,
    pub src_reg: RegAddr,
    pub imm: LC3Word,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct InstrRegReg {
    pub dest_reg: RegAddr,
    pub src_reg_1: RegAddr,
    pub src_reg_2: RegAddr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct InstrRegOnly {
    pub dest_reg: RegAddr,
    pub src_reg: RegAddr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct InstrOffset6 {
    pub target_reg: RegAddr,
    pub base_reg: RegAddr,
    pub offset: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct InstrPCOffset9 {
    pub target_reg: RegAddr,
    pub pc_offset: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct InstrPCOffset11 {
    pub pc_offset: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IAdd {
    Reg(InstrRegReg),
    Imm(InstrRegImm),
}
const ADD_OPCODE: u8 = 0b0001;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IAnd {
    Reg(InstrRegReg),
    Imm(InstrRegImm),
}
const AND_OPCODE: u8 = 0b0101;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct INot(InstrRegOnly);
const NOT_OPCODE: u8 = 0b1001;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConditionCodes {
    pub positive: bool,
    pub negative: bool,
    pub zero: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct IBranch {
    //while br roughly follows the bit assignment of PCoffset9,
    //this is treated as a special case for ease of implementation
    pub cond_codes: ConditionCodes,
    pub pc_offset: u16,
}
const BRANCH_OPCODE: u8 = 0b0000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IJump {
    Instr(RegAddr), //not strictly an offset6, but doesn't matter here
    Ret,            //RET and RETI are included here, as they are functionally special cases of JMP
    InterRet,
}
const JMP_OPCODE: u8 = 0b1100;
const RTI_OPCODE: u8 = 0b1000;
const ALL_JUMP_OPCODES: [u8; 2] = [JMP_OPCODE, RTI_OPCODE];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IJumpSubRoutine {
    Offset(InstrPCOffset11), //JSR
    Reg(RegAddr),            //JSRR treated as an offset6 with an offset of 0
}
const JSR_OPCODE: u8 = 0b0100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ILoad {
    Std(InstrPCOffset9),      //LD
    Indirect(InstrPCOffset9), //LDI
    Reg(InstrOffset6),        //LDR
    Addr(InstrPCOffset9),     //LEA
}
const LD_OPCODE: u8 = 0b0010;
const LDI_OPCODE: u8 = 0b1010;
const LDR_OPCODE: u8 = 0b0110;
const LEA_OPCODE: u8 = 0b1110;
const ALL_LOAD_OPCODES: [u8; 4] = [LD_OPCODE, LDI_OPCODE, LDR_OPCODE, LEA_OPCODE];

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Trap {
    Getc, // 0x20
    Out,  // 0x21
    PutS, // 0x22
    In,   // 0x23
    Halt, // 0x24
}
const TRAP_OPCODE: u8 = 0b1111;

/// Set the processor condition codes from `result`.
fn set_condition_codes<P: LC3>(processor: &mut P, result: LC3Word) {
    match (result as SignedLC3Word).cmp(&0) {
        std::cmp::Ordering::Greater => processor.flag_positive(),
        std::cmp::Ordering::Less => processor.flag_negative(),
        std::cmp::Ordering::Equal => processor.flag_zero(),
    }
}

/// Parses the opcode from a word.
#[inline]
const fn get_opcode(word: LC3Word) -> u8 {
    // Opcode is always the top byte
    word.to_be_bytes()[0]
}

/// Extracts a range of bits from a word.
///
/// `start`, `end` is inclusive.
/// May panic or return undefined output if:
/// * `start` >= LC3Word length
/// * `end` >= LC3Word length
/// * `end` > start
///
/// e.g. get_bits(0x00A2, 5, 1)
/// * 0x00A2 -> 0000 0000 1010 0010
/// * 0000 0000 10[10 001]0
/// * return 0x0011 -> 1 0001
#[inline]
const fn get_bits(mut word: LC3Word, start: u8, end: u8) -> u16 {
    const WORD_BITS: u8 = LC3Word::BITS as u8;
    const WORD_SHIFT_START: u8 = WORD_BITS - 1;

    debug_assert!(start < WORD_BITS);
    debug_assert!(end <= start);

    // Set bits above `start` to zero.
    let shift_out_top = WORD_SHIFT_START - start;
    word <<= shift_out_top;
    word >>= shift_out_top;

    // Remove bits below `end` to zero.
    word >>= end;

    // Only the given range remains in the word
    word
}

/// Gets a single bit from a word.
///
/// May panic or return undefined output if
/// `loc` >= LC3Word length
///
/// e.g. get_bit(0x00A2, 2)
/// * 0x00A2 -> 0000 0000 1010 0010
/// * 0000 0000 1010 0[0]10
/// * return 0x0000
#[inline]
const fn get_bit(word: LC3Word, loc: u8) -> u8 {
    get_bits(word, loc, loc) as u8
}

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

impl Instruction for IAnd {
    fn execute<P: LC3>(self, processor: &mut P) {
        let dest;
        let result = match self {
            Self::Reg(InstrRegReg {
                dest_reg,
                src_reg_1,
                src_reg_2,
            }) => {
                dest = dest_reg;
                processor.reg(src_reg_1) & processor.reg(src_reg_2)
            }
            Self::Imm(InstrRegImm {
                dest_reg,
                src_reg,
                imm,
            }) => {
                dest = dest_reg;
                processor.reg(src_reg) & imm
            }
        };
        processor.set_reg(dest, result);
        set_condition_codes(processor, result);
    }

    fn parse(word: LC3Word) -> Option<Self>
    where
        Self: Sized,
    {
        if get_opcode(word) == AND_OPCODE {
            // 3 bits is always a valid RegAddr
            let dest_reg = RegAddr::panic_from_u16(get_bits(word, 11, 9));
            // 3 bits is always a valid RegAddr
            let src_reg_1 = RegAddr::panic_from_u16(get_bits(word, 8, 6));

            if get_bit(word, 5) == 0 {
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

impl Instruction for IBranch {
    fn execute<P: LC3>(self, processor: &mut P) {
        let pos_condition = self.cond_codes.positive && processor.positive_cond();
        let zero_condition = self.cond_codes.zero && processor.zero_cond();
        let neg_condition = self.cond_codes.negative && processor.negative_cond();

        if pos_condition || zero_condition || neg_condition {
            processor.set_pc(processor.pc() + self.pc_offset);
        }
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

impl Instruction for IJumpSubRoutine {
    fn execute<P: LC3>(self, processor: &mut P) {
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
