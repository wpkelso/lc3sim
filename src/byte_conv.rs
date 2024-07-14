use std::{
    error::Error,
    fmt::{Debug, Display},
};

use crate::{AsmLine, IAdd, IAnd, InstEnum, InstrImm, InstrReg};

pub fn to_3_bits(val: u8, offset: u8) -> u16 {
    ((val & 0b0111) as u16) << offset
}

pub fn get_3_bits(inst: u16, offset: u8) -> u8 {
    ((inst >> offset) & 0b0111) as u8
}

/// Convert `val` to a 5-bit signed immediate value.
pub fn to_imm(val: i16) -> u16 {
    (val & 0b1_1111) as u16
}

/// Get an immediate value from the last 5 bits.
pub fn get_imm(inst: u16) -> i16 {
    // Sign extension on 5-width to 8-width
    const LEADING_ZERO_WIDTH: u8 = 8 + 3;
    (((inst & 0b1_1111) << LEADING_ZERO_WIDTH) as i16) >> LEADING_ZERO_WIDTH
}

/// Check the 5th bit for an immediate value.
pub fn is_imm(inst: u16) -> bool {
    ((inst >> 5) & 0b1) != 0
}

impl From<AsmLine> for IAdd {
    fn from(value: AsmLine) -> Self {
        let dr = get_3_bits(value, 9);
        let reg = get_3_bits(value, 6);
        if is_imm(value) {
            let imm = get_imm(value);
            Self::Imm(InstrImm { dr, reg, imm })
        } else {
            let reg2 = get_3_bits(value, 0);
            Self::Reg(InstrReg {
                dr,
                reg1: reg,
                reg2,
            })
        }
    }
}

impl From<IAdd> for AsmLine {
    fn from(val: IAdd) -> Self {
        const TAG: u16 = 0b0001 << 12;
        let mut bits: u16 = TAG;

        match val {
            IAdd::Imm(InstrImm { dr, reg, imm }) => {
                bits |= to_3_bits(dr, 9);
                bits |= to_3_bits(reg, 6);
                bits |= 0b10_0000;
                bits |= to_imm(imm)
            }
            IAdd::Reg(InstrReg { dr, reg1, reg2 }) => {
                bits |= to_3_bits(dr, 9);
                bits |= to_3_bits(reg1, 6);
                bits |= to_3_bits(reg2, 0);
            }
        }
        bits
    }
}

impl From<AsmLine> for IAnd {
    fn from(_value: AsmLine) -> Self {
        todo!()
    }
}

impl From<IAnd> for AsmLine {
    fn from(_val: IAnd) -> Self {
        todo!()
    }
}

pub struct InvalidTag {
    tag: u8,
}

impl Display for InvalidTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "Tag {:#01x} is not a valid LC-3 opcode.",
            self.tag
        ))
    }
}

impl Debug for InvalidTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Display>::fmt(self, f)
    }
}

impl Error for InvalidTag {}

/// Creates the appropriate [`Instruction`] based on its tag.
impl TryFrom<AsmLine> for InstEnum {
    type Error = InvalidTag;
    fn try_from(value: AsmLine) -> Result<Self, InvalidTag> {
        let tag = (value >> 12) & 0b1111;
        match tag {
            0b0001 => Ok(InstEnum::IAdd(IAdd::from(value))),
            0b0101 => Ok(InstEnum::IAnd(IAnd::from(value))),
            invalid => Err(InvalidTag { tag: invalid as u8 }),
        }
    }
}

/// Matches the appropriate `From<X> for AsmLine` in members.
impl From<InstEnum> for AsmLine {
    fn from(val: InstEnum) -> Self {
        match val {
            InstEnum::IAdd(x) => Self::from(x),
            InstEnum::IAnd(x) => Self::from(x),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn iadd_parse() {
        const EXPECTED: &[IAdd] = &[
            IAdd::Imm(InstrImm {
                dr: 5,
                reg: 3,
                imm: -2,
            }),
            IAdd::Imm(InstrImm {
                dr: 2,
                reg: 1,
                imm: 2,
            }),
            IAdd::Reg(InstrReg {
                dr: 4,
                reg1: 1,
                reg2: 7,
            }),
        ];
        #[allow(clippy::unusual_byte_groupings)]
        const VALUES: [u16; 3] = [
            0b0001_101_011_1_11110,
            0b0001_010_001_1_00010,
            0b0001_100_001_000_111,
        ];

        let parsed = VALUES.map(IAdd::from);
        assert_eq!(EXPECTED, parsed);

        let byte_converted = parsed.map(|v| v.into());
        assert_eq!(VALUES, byte_converted);
    }

    #[test]
    fn enum_parse() {
        const EXPECTED: [IAdd; 3] = [
            IAdd::Imm(InstrImm {
                dr: 5,
                reg: 3,
                imm: -2,
            }),
            IAdd::Imm(InstrImm {
                dr: 2,
                reg: 1,
                imm: 2,
            }),
            IAdd::Reg(InstrReg {
                dr: 4,
                reg1: 1,
                reg2: 7,
            }),
        ];
        #[allow(clippy::unusual_byte_groupings)]
        const VALUES: [u16; 3] = [
            0b0001_101_011_1_11110,
            0b0001_010_001_1_00010,
            0b0001_100_001_000_111,
        ];

        let expected_enum = EXPECTED.map(InstEnum::from);

        let parsed = VALUES.map(|v| InstEnum::try_from(v).unwrap());
        assert_eq!(expected_enum, parsed);
        assert_eq!(EXPECTED, parsed.map(|v| v.try_into().unwrap()));

        let byte_converted = parsed.map(|v| v.into());
        assert_eq!(VALUES, byte_converted);
    }
}
