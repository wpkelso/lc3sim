use std::io::Read;

use thiserror::Error;

use crate::{
    defs::{LC3MemAddr, LC3Word, RegAddr, STACK_REG},
    instruction::{Instruction, InstructionEnum, InstructionErr, InsufficientPerms},
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
    #[error(transparent)]
    InsufficientPerms(InsufficientPerms),
    #[error("{max_addr} is the largest possible LC3 address, PC cannot advance further.", max_addr = LC3MemAddr::MAX)]
    LastAddress,
    #[error("The machine must be unhalted to progress")]
    Halted,
    #[error("The MCR control bit was cleared, disabling the clock")]
    ClockDisabled,
}

impl From<InstructionErr> for StepFailure {
    fn from(value: InstructionErr) -> Self {
        match value {
            InstructionErr::InsufficientPerms(x) => Self::InsufficientPerms(x),
        }
    }
}

// LC3 condition mask/shift consts
const PRIV_MASK: LC3Word = 1 << 15;
const PRIORITY_SHIFT: LC3Word = 8;
const PRIORITY_MASK: LC3Word = 111 << PRIORITY_SHIFT;
const NEGATIVE_MASK: LC3Word = 1 << 2;
const ZERO_MASK: LC3Word = 1 << 1;
const POSITIVE_MASK: LC3Word = 1;

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
        let privilege = if self.privileged() { 0 } else { PRIV_MASK };
        let with_priority = privilege | ((self.priority() as LC3Word) << PRIORITY_SHIFT);

        let n = if self.negative_cond() {
            NEGATIVE_MASK
        } else {
            0
        };
        let z = if self.zero_cond() { ZERO_MASK } else { 0 };
        let p = if self.positive_cond() {
            POSITIVE_MASK
        } else {
            0
        };

        with_priority | n | z | p
    }

    /// Restores the processor status register value.
    fn set_processor_status_reg(&mut self, status_reg: LC3Word) {
        self.set_privileged((status_reg & PRIV_MASK) == 0);
        self.set_priority(((status_reg & PRIORITY_MASK) >> PRIORITY_SHIFT) as u8);

        if (status_reg & NEGATIVE_MASK) != 0 {
            self.flag_positive();
        } else if (status_reg & ZERO_MASK) != 0 {
            self.flag_zero();
        } else if (status_reg & POSITIVE_MASK) != 0 {
            self.flag_positive();
        } else {
            self.clear_flags();
        }
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
    ///
    /// `set_priority` is `Some` on I/O device interrupts, `None` on exceptions.
    fn interrupt(&mut self, vector: LC3Word, set_priority: Option<u8>) {
        let psr = self.processor_status_reg();

        self.set_privileged(true);

        if let Some(priority) = set_priority {
            self.set_priority(priority);
        }

        // PSR and PC stack pushes
        let stack_reg = self.reg(STACK_REG);
        self.set_mem(stack_reg, psr);
        self.set_mem(stack_reg - 1, self.pc());
        self.set_reg(STACK_REG, stack_reg - 2);

        self.set_pc(0x0100 + vector);
    }

    /// Fill the lines from `start` with `words`.
    fn populate<I: IntoIterator<Item = LC3Word>>(&mut self, start: LC3MemAddr, words: I);
}

/// Populates the processor from a binary provider.
///
/// Invalid binary data is silently discarded.
pub fn populate_from_bin<P: LC3, R: Read>(processor: &mut P, bin: R) {
    let mut bytes = bin.bytes();

    let mut next_pair = || {
        let first = bytes.next()?.ok()?;
        let second = bytes.next()?.ok()?;
        Some(LC3Word::from_be_bytes([first, second]))
    };

    if let Some(start) = next_pair() {
        processor.populate(start, std::iter::from_fn(next_pair));
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::executors::core::CoreLC3;

    #[test]
    fn psr_set() {
        let mut processor = CoreLC3::new();

        processor.set_priority(0);
        processor.set_privileged(true);
        processor.clear_flags();

        assert_eq!(processor.processor_status_reg(), 0x0000);

        processor.set_privileged(false);
        assert_eq!(processor.processor_status_reg(), 0x8000);

        processor.flag_negative();
        assert_eq!(processor.processor_status_reg(), 0x8004);

        processor.flag_zero();
        assert_eq!(processor.processor_status_reg(), 0x8002);

        processor.flag_positive();
        assert_eq!(processor.processor_status_reg(), 0x8001);

        processor.set_priority(5);
        assert_eq!(processor.processor_status_reg(), 0x8501);
    }

    #[test]
    fn psr_recover() {
        let mut processor = CoreLC3::new();
        const PSR_VAL: LC3Word = 0x8202;
        const PRIORITY: u8 = 2;

        processor.set_priority(PRIORITY);
        processor.set_privileged(false);
        processor.clear_flags();
        processor.flag_zero();

        assert_eq!(processor.processor_status_reg(), PSR_VAL);

        processor.set_processor_status_reg(0);
        assert_eq!(processor.processor_status_reg(), 0x0000);
        assert!(processor.privileged());
        assert_eq!(processor.priority(), 0);
        assert!(!processor.negative_cond());
        assert!(!processor.zero_cond());
        assert!(!processor.positive_cond());

        processor.set_processor_status_reg(PSR_VAL);
        assert_eq!(processor.processor_status_reg(), PSR_VAL);
        assert!(!processor.privileged());
        assert_eq!(processor.priority(), PRIORITY);
        assert!(!processor.negative_cond());
        assert!(processor.zero_cond());
        assert!(!processor.positive_cond());
    }
}
