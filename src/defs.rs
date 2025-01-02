use thiserror::Error;
use strum_macros::EnumDiscriminants;

pub type LC3Word = u16;
pub type SignedLC3Word = i16;
pub type LC3MemAddr = u16;

pub const ADDR_SPACE_SIZE: usize = 2_usize.pow(16_u32); //size of the memory address space
pub const TRAP_VEC_TBL: LC3Word = 0x0000; //first address of the trap vector table
pub const IR_VEC_TBL: LC3Word = 0x0100; //first address of the interrupt vector table
pub const OS_SUPER_STACK: LC3Word = 0x0200; //first address of the operating and supervisor
                                            //stack space
pub const USER_SPACE: LC3Word = 0x3000; //first address of the user code space
pub const DEV_REG_ADDR: LC3Word = 0xFE00; //first address of the device register address
                                          //space

pub const NUM_REGS: usize = 8_usize; //number of registers in the LC3 spec

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RegAddr {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
}

impl RegAddr {
    /// `const` optimized u8 conversion.
    ///
    /// Panics when u8 is out of range.
    pub const fn panic_from_u8(value: u8) -> Self {
        match value {
            0 => RegAddr::Zero,
            1 => RegAddr::One,
            2 => RegAddr::Two,
            3 => RegAddr::Three,
            4 => RegAddr::Four,
            5 => RegAddr::Five,
            6 => RegAddr::Six,
            7 => RegAddr::Seven,
            _ => panic!("Argument outside of [0, 7] (the valid LC-3 registers"),
        }
    }

    /// Converts and calls [`Self::panic_from_u8`].
    pub const fn panic_from_u16(value: u16) -> Self {
        Self::panic_from_u8(value as u8)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Error)]
#[error("{0} is not in [0, 7] (the valid LC-3 registers)")]
pub struct InvalidRegAddr(u8);

impl TryFrom<u8> for RegAddr {
    type Error = InvalidRegAddr;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(RegAddr::Zero),
            1 => Ok(RegAddr::One),
            2 => Ok(RegAddr::Two),
            3 => Ok(RegAddr::Three),
            4 => Ok(RegAddr::Four),
            5 => Ok(RegAddr::Five),
            6 => Ok(RegAddr::Six),
            7 => Ok(RegAddr::Seven),
            x => Err(InvalidRegAddr(x)),
        }
    }
}

impl From<RegAddr> for u8 {
    fn from(value: RegAddr) -> Self {
        match value {
            RegAddr::Zero => 0,
            RegAddr::One => 1,
            RegAddr::Two => 2,
            RegAddr::Three => 3,
            RegAddr::Four => 4,
            RegAddr::Five => 5,
            RegAddr::Six => 6,
            RegAddr::Seven => 7,
        }
    }
}

impl From<RegAddr> for usize {
    fn from(value: RegAddr) -> Self {
        u8::from(value) as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, EnumDiscriminants)]
pub enum Op {
    ADD,
    AND,
    BR(bool, bool, bool), // NZP
    JMP,
    JSR,
    JSRR,
    LD,
    LDI,
    LDR,
    LEA,
    NOT,
    RET,
    RTI,
    ST,
    STI,
    STR,
    TRAP,
    GETC,
    OUT,
    PUTS,
    IN,
    PUTSP,
    HALT,
    ILLEGAL,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, EnumDiscriminants)]
pub enum PseudoOp {
    ORIG,
    FILL,
    BLKW,
    STRINGZ,
    END,
    ILLEGAL,
}
