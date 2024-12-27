use std::future::Future;

use thiserror::Error;

use crate::{
    defs::{LC3MemAddr, LC3Word, RegAddr},
    instruction::{Instruction, InstructionEnum},
    util::format_word_bits,
};

pub mod core;

/// LC3 Memory Address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LC3MemLoc {
    pub loc: LC3MemAddr,
    pub value: LC3Word,
}

/// Failure occured during a machine step.
///
/// [`Self::InvalidInstruction`] and [`Self::InsufficientPerms`] enter an
/// exception.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Error)]
pub enum StepFailure {
    #[error(
        "{0} is not a valid LC3 instruction: {top_bits} is an invalid opcode.", top_bits = format_word_bits(*.0, 0)
    )]
    InvalidInstruction(LC3Word),
    #[error("User mode attempted supervisor mode operation.")]
    InsufficientPerms,
    #[error("{max_addr} is the largest possible LC3 address, PC cannot advance further.", max_addr = LC3MemAddr::MAX)]
    LastAddress,
    #[error("The machine must be unhalted to progress")]
    Halted,
}

/// Full LC3 simulator.
pub trait LC3 {
    /// Current program counter.
    fn pc(&self) -> LC3MemAddr;
    /// Replace the current program counter.
    fn set_pc(&mut self, pc: LC3MemAddr);

    fn reg(&self, addr: RegAddr) -> LC3Word;
    fn set_reg(&mut self, addr: RegAddr, value: LC3Word);

    fn mem(&self, addr: LC3MemAddr) -> LC3Word;
    fn set_mem(&mut self, addr: LC3MemAddr, value: LC3Word);

    /// Current priority in [0, 7].
    ///
    /// 0 is the lowest priority, 7 is the highest.
    fn priority(&self) -> u8;
    /// Sets priority if in [0, 7].
    fn set_priority(&mut self, priority: u8);

    /// True if in supervisor mode, false if in user mode.
    fn privileged(&self) -> bool;
    /// Sets to supervisor mode if true; to user mode if false.
    fn set_privileged(&mut self, priviledged: bool);

    /// Returns the current processor status register value.
    fn processor_status_reg(&self) -> LC3Word {
        let privilege = if self.privileged() { 0 } else { 1 << 15 };
        let with_priority = privilege | ((self.priority() as LC3Word) << 8);

        let n = if self.negative_cond() { 1 << 2 } else { 0 };
        let z = if self.zero_cond() { 1 << 1 } else { 0 };
        let p = if self.positive_cond() { 1 } else { 0 };

        with_priority | n | z | p
    }

    /// Return the instruction at [`Self::pc`], if any.
    fn cur_inst(&self) -> Option<InstructionEnum> {
        InstructionEnum::parse(self.mem(self.pc()))
    }

    /// Returns true if the positive flag is set.
    fn positive_cond(&self) -> bool;
    /// Returns true if the zero flag is set.
    fn zero_cond(&self) -> bool;
    /// Returns true if the negative flag is set.
    fn negative_cond(&self) -> bool;

    /// Sets the positive flag.
    fn flag_positive(&mut self);
    /// Sets the zero flag.
    fn flag_zero(&mut self);
    /// Sets the negative flag.
    fn flag_negative(&mut self);

    /// Clears the sign flags.
    fn clear_flags(&mut self);

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
    /// Unset the machine from a halted state, if set.
    fn unhalt(&mut self);
    /// Check if the machine is set to halted.
    fn is_halted(&self) -> bool;

    /// Processes the instruction at [`Self::pc`].
    fn step(&mut self) -> Result<(), StepFailure>;

    /// Initiates the interrupt service routine for `vector`.
    fn interrupt(&mut self, vector: LC3Word) -> Result<(), StepFailure> {
        todo!()
    }

    /// Fill the lines from `start` with `words`.
    fn populate<I: IntoIterator<Item = LC3Word>>(&mut self, start: LC3MemAddr, words: I);
}
