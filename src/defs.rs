use thiserror::Error;

pub type LC3Word = u16;
pub type SignedLC3Word = i16;
pub type LC3MemAddr = u16;

/// Size of the memory address space.
pub const ADDR_SPACE_SIZE: usize = 2_usize.pow(16_u32);
/// First address of the trap vector table.
pub const TRAP_VEC_TBL: LC3Word = 0x0000;
/// First address of the interrupt vector table.
pub const IR_VEC_TBL: LC3Word = 0x0100;
/// First address of the operating and supervisor stack space.
pub const OS_SUPER_STACK: LC3Word = 0x0200;
/// First address of the user code space.
pub const USER_SPACE: LC3Word = 0x3000;
/// First address of the device register address space.
pub const DEV_REG_ADDR: LC3Word = 0xFE00;

/// Initial supervisor stack pointer value.
pub const SUPERVISOR_SP_INIT: LC3Word = USER_SPACE - 1;

/// Vector for a keyboard I/O interrupt.
pub const KEYBOARD_INTERRUPT: LC3Word = 0x0080;
/// Priority for an I/O interrupt
pub const IO_PRIORITY: u8 = 4;

/// Register with a special stack meaning in privileged mode.
pub const STACK_REG: RegAddr = RegAddr::Six;

/// Bit 15 is 1 when the keyboard has received a new character.
pub const KEYBOARD_STATUS_REGISTER: LC3Word = 0xFE00;
/// Last character typed on the keyboard.
pub const KEYBOARD_DATA_REGISTER: LC3Word = 0xFE02;
/// Bit 15 is 1 when the display is ready to receive a new character.
pub const DISPLAY_STATUS_REGISTER: LC3Word = 0xFE04;
/// Characters written to this low byte will be displayed on screen.
pub const DISPLAY_DATA_REGISTER: LC3Word = 0xFE06;
/// Bit 15 is clock enable -- machine runs while set to 1.
pub const MACHINE_CONTROL_REGISTER: LC3Word = 0xFFFE;

/// Number of registers in the LC3 spec
pub const NUM_REGS: usize = 8_usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

impl From<RegAddr> for u16 {
    fn from(value: RegAddr) -> Self {
        u8::from(value) as u16
    }
}
