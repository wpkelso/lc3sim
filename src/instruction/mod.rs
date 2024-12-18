//TODO: TRAP instructions

use crate::{
    defs::{LC3Word, SignedLC3Word},
    executors::LC3,
};

mod args;
pub use args::*;

mod iadd;
pub use iadd::IAdd;
mod iand;
pub use iand::IAnd;
mod inot;
pub use inot::INot;
mod ibranch;
pub use ibranch::IBranch;
mod ijump;
pub use ijump::IJump;
mod ijumpsr;
pub use ijumpsr::IJumpSubRoutine;
mod iload;
pub use iload::ILoad;
mod istore;
pub use istore::IStore;
mod trap;
pub use trap::Trap;

pub trait Instruction {
    /// Run this instruction on `P`, producing all outputs and side effects.
    fn execute<P: LC3>(self, processor: &mut P);

    /// Convert the word into this instruction, if possible.
    fn parse(word: LC3Word) -> Option<Self>
    where
        Self: Sized;
}

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
