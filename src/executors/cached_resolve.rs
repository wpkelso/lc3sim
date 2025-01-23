use std::{iter::FusedIterator, pin::Pin};

use pinned_init::{init, pin_data, pin_init, InPlaceInit};

use crate::{
    defs::{
        LC3MemAddr, LC3Word, RegAddr, ADDR_SPACE_SIZE, MACHINE_CONTROL_REGISTER, NUM_REGS,
        OS_SUPER_STACK, STACK_REG, SUPERVISOR_SP_INIT,
    },
    instruction::{Instruction, InstructionEnum},
};

use super::{LC3MemLoc, StepFailure, LC3};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Condition {
    None,
    Negative,
    Zero,
    Positive,
}

/// Caches a resolution of LC3Word to an instruction
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum CachedResolve {
    Raw(LC3Word),
    Resolved(InstructionEnum),
}

impl CachedResolve {
    pub fn new(word: LC3Word) -> Self {
        Self::Raw(word)
    }

    #[inline]
    #[cold]
    pub fn raw_instr(&mut self, word: LC3Word) -> Option<InstructionEnum> {
        let instr = InstructionEnum::parse(word)?;
        *self = Self::Resolved(instr);
        Some(instr)
    }

    pub fn instr(&mut self) -> Option<InstructionEnum> {
        match *self {
            Self::Raw(word) => self.raw_instr(word),
            Self::Resolved(instr) => Some(instr),
        }
    }

    pub fn word(&self) -> LC3Word {
        match self {
            Self::Raw(word) => *word,
            Self::Resolved(instr) => (*instr).into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[pin_data]
#[repr(transparent)]
struct CachedResolveStruct {
    inner: CachedResolve,
}

impl CachedResolveStruct {
    pub fn new(word: LC3Word) -> Self {
        Self {
            inner: CachedResolve::new(word),
        }
    }

    pub fn instr(&mut self) -> Option<InstructionEnum> {
        self.inner.instr()
    }

    pub fn word(&self) -> LC3Word {
        self.inner.word()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[pin_data]
pub struct CachedResolveLC3 {
    mem: [CachedResolveStruct; ADDR_SPACE_SIZE],
    condition: Condition,
    priority: u8,
    privileged: bool,
    regs: [LC3Word; NUM_REGS],
    supervisor_sp: LC3Word,
    pc: LC3MemAddr,
    halted: bool,
    mpr_disabled: bool,
}

impl CachedResolveLC3 {
    pub fn new() -> Self {
        Self {
            mem: [CachedResolveStruct::new(0); ADDR_SPACE_SIZE],
            condition: Condition::None,
            priority: 0,
            privileged: true,
            supervisor_sp: SUPERVISOR_SP_INIT,
            regs: [0; NUM_REGS],
            pc: OS_SUPER_STACK,
            halted: false,
            mpr_disabled: false,
        }
    }

    /// Constructs this as a box without allocating the full size on stack.
    pub fn boxed() -> Box<Self> {
        let this = Box::pin_init(pin_init!(Self {
            // This is the field that might exceed stack capacity
            mem <- pinned_init::init_array_from_fn(|_| init!(CachedResolveStruct {
                inner: CachedResolve::Raw(0)
            })),
            condition: Condition::None,
            priority: 0,
            privileged: true,
            supervisor_sp: SUPERVISOR_SP_INIT,
            regs: [0; NUM_REGS],
            pc: OS_SUPER_STACK,
            halted: false,
            mpr_disabled: false,
        }));
        Pin::into_inner(this.expect("Infalliable"))
    }
}

impl Default for CachedResolveLC3 {
    fn default() -> Self {
        Self::new()
    }
}

impl LC3 for CachedResolveLC3 {
    fn pc(&self) -> LC3MemAddr {
        self.pc
    }
    fn set_pc(&mut self, pc: LC3MemAddr) {
        self.pc = pc
    }

    fn reg(&self, addr: RegAddr) -> LC3Word {
        let reg_addr = usize::from(addr);

        if self.privileged && reg_addr == STACK_REG.into() {
            self.supervisor_sp
        } else {
            self.regs[reg_addr]
        }
    }
    fn set_reg(&mut self, addr: RegAddr, value: LC3Word) {
        let reg_addr = usize::from(addr);

        if self.privileged && reg_addr == STACK_REG.into() {
            self.supervisor_sp = value
        } else {
            self.regs[reg_addr] = value
        }
    }

    fn mem(&self, addr: LC3MemAddr) -> LC3Word {
        self.mem[addr as usize].word()
    }
    fn set_mem(&mut self, addr: LC3MemAddr, value: LC3Word) {
        self.mem[addr as usize] = CachedResolveStruct::new(value);
        if addr == MACHINE_CONTROL_REGISTER {
            self.mpr_disabled = (value & (1 << 15)) == 0;
        }
    }

    fn priority(&self) -> u8 {
        self.priority
    }
    fn set_priority(&mut self, priority: u8) {
        if priority < 8 {
            self.priority = priority
        }
    }

    fn privileged(&self) -> bool {
        self.privileged
    }
    fn set_privileged(&mut self, priviledged: bool) {
        self.privileged = priviledged
    }

    fn positive_cond(&self) -> bool {
        self.condition == Condition::Positive
    }
    fn zero_cond(&self) -> bool {
        self.condition == Condition::Zero
    }
    fn negative_cond(&self) -> bool {
        self.condition == Condition::Negative
    }

    fn flag_positive(&mut self) {
        self.condition = Condition::Positive;
    }
    fn flag_zero(&mut self) {
        self.condition = Condition::Zero;
    }
    fn flag_negative(&mut self) {
        self.condition = Condition::Negative;
    }

    fn clear_flags(&mut self) {
        self.condition = Condition::None;
    }

    type FullIter<'a> = CachedLC3FullIter<'a>;
    fn iter(&self) -> Self::FullIter<'_> {
        CachedLC3FullIter::new(self)
    }

    type SparseIter<'a> = CachedLC3SparseIter<'a>;
    fn sparse_iter(&self) -> Self::SparseIter<'_> {
        CachedLC3SparseIter::new(self.iter())
    }

    fn halt(&mut self) {
        self.halted = true;
    }

    fn unhalt(&mut self) {
        self.halted = false;
    }

    fn is_halted(&self) -> bool {
        self.halted
    }

    /// Executes the current instruction.
    ///
    /// Does not handle memory map updates.
    fn step(&mut self) -> Result<(), StepFailure> {
        if self.halted {
            Err(StepFailure::Halted)
        } else if self.mpr_disabled {
            Err(StepFailure::ClockDisabled)
        } else {
            let inst = self.mem[self.pc() as usize]
                .instr()
                .ok_or(StepFailure::InvalidInstruction(self.mem(self.pc())))?;

            inst.execute(self)?;

            if !matches!(inst, InstructionEnum::IBranch(_))
                && !matches!(inst, InstructionEnum::IJump(_))
            {
                self.pc += 1;
            }

            Ok(())
        }
    }

    fn populate<I: IntoIterator<Item = LC3Word>>(&mut self, start: LC3MemAddr, words: I) {
        let mem_iter_mut = self.mem[start.into()..].iter_mut();
        for (word, loc) in words.into_iter().zip(mem_iter_mut) {
            *loc = CachedResolveStruct::new(word);
        }
    }
}

/// Sparse iterator for [`CoreLC3`].
///
/// Skips all zero elements.
pub struct CachedLC3FullIter<'a> {
    iter: std::slice::Iter<'a, CachedResolveStruct>,
}

impl<'a> CachedLC3FullIter<'a> {
    fn new(base: &'a CachedResolveLC3) -> Self {
        Self {
            iter: base.mem.iter(),
        }
    }
}

impl Iterator for CachedLC3FullIter<'_> {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|x| x.word())
    }
}

impl DoubleEndedIterator for CachedLC3FullIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|x| x.word())
    }
}

impl ExactSizeIterator for CachedLC3FullIter<'_> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl FusedIterator for CachedLC3FullIter<'_> {}

/// Sparse iterator for [`CoreLC3`].
///
/// Skips all zero elements.
pub struct CachedLC3SparseIter<'a> {
    iter: std::iter::Enumerate<CachedLC3FullIter<'a>>,
}

impl<'a> CachedLC3SparseIter<'a> {
    fn new(iter: <CachedResolveLC3 as LC3>::FullIter<'a>) -> Self {
        Self {
            iter: iter.enumerate(),
        }
    }
}

impl Iterator for CachedLC3SparseIter<'_> {
    type Item = LC3MemLoc;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (loc, value) = self.iter.next()?;
            if value != 0 {
                return Some(LC3MemLoc {
                    loc: loc as u16,
                    value,
                });
            }
        }
    }
}

impl DoubleEndedIterator for CachedLC3SparseIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            let (loc, value) = self.iter.next_back()?;
            if value != 0 {
                return Some(LC3MemLoc {
                    loc: loc as u16,
                    value,
                });
            }
        }
    }
}

impl FusedIterator for CachedLC3SparseIter<'_> {}
