use thiserror::Error;

use crate::{
    defs::{LC3MemAddr, LC3Word, RegAddr},
    util::format_word_bits,
};

pub mod core;

/// LC3 Memory Address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LC3MemLoc {
    pub loc: LC3MemAddr,
    pub value: LC3Word,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Error)]
pub enum StepFailure {
    #[error(
        "{0} is not a valid LC3 instruction: {top_bits} is an invalid opcode.", top_bits = format_word_bits(*.0, 0)
    )]
    InvalidInstruction(LC3Word),
    #[error("{max_addr} is the largest possible LC3 address, PC cannot advance further.", max_addr = LC3MemAddr::MAX)]
    LastAddress,
    #[error("The machine must be unhalted to progress (may be in TRAP for I/O)")]
    Halted,
}

/// Full LC3 simulator.
pub trait LC3 {
    /// Current program counter
    fn pc(&self) -> LC3MemAddr;
    /// Replace the current program counter
    fn set_pc(&mut self, pc: LC3MemAddr);

    fn reg(&self, addr: RegAddr) -> LC3Word;
    fn set_reg(&mut self, addr: RegAddr, value: LC3Word);

    fn mem(&self, addr: LC3MemAddr) -> LC3Word;
    fn set_mem(&mut self, addr: LC3MemAddr, value: LC3Word);

    /// Returns true if the positive flag is set
    fn positive_cond(&self) -> bool;
    /// Returns true if the zero flag is set
    fn zero_cond(&self) -> bool;
    /// Returns true if the negative flag is set
    fn negative_cond(&self) -> bool;

    /// Sets the positive flag
    fn flag_positive(&mut self);
    /// Sets the zero flag
    fn flag_zero(&mut self);
    /// Sets the negative flag
    fn flag_negative(&mut self);

    /// Produces all words in order from 0x0000.
    type FullIter<'a>: Iterator<Item = LC3Word>
    where
        Self: 'a;
    fn iter(&self) -> Self::FullIter<'_>;

    /// Produces all words in order from 0x0000, possibly skipping 0x0000 words.
    type SparseIter<'a>: Iterator<Item = LC3MemLoc>
    where
        Self: 'a;
    fn sparse_iter(&self) -> Self::SparseIter<'_>;

    /// Set the machine to a halted state.
    fn halt(&mut self);
    /// Clear a halted state, if any.
    fn unhalt(&mut self);
    /// Check if the machine is set to halted.
    fn is_halted(&self) -> bool;

    fn step(&mut self) -> Result<(), StepFailure>;

    /// Fill the lines from `start` with `words`.
    fn populate<I: IntoIterator<Item = LC3Word>>(&mut self, start: LC3MemAddr, words: I);
}
