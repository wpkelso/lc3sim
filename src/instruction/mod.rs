//TODO: TRAP instructions

use crate::{defs::LC3Word, executors::LC3};

mod args;
pub use args::*;
mod util;
use iadd::ADD_OPCODE;
use iand::AND_OPCODE;
use ibranch::BRANCH_OPCODE;
use ijump::ALL_JUMP_OPCODES;
use ijumpsr::JSR_OPCODE;
use iload::ALL_LOAD_OPCODES;
use inot::NOT_OPCODE;
use istore::ALL_STORE_OPCODES;
use trap::TRAP_OPCODE;
use util::*;

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

/// Captures all LC-3 [`Instruction`]s.
///
/// If this does not parse and execute, there is no valid LC-3 instruction
/// that could parse and execute the given word.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum InstructionEnum {
    IAdd(IAdd),
    IAnd(IAnd),
    INot(INot),
    IBranch(IBranch),
    IJump(IJump),
    IJumpSubRoutine(IJumpSubRoutine),
    ILoad(ILoad),
    IStore(IStore),
    Trap(Trap),
}

impl Instruction for InstructionEnum {
    fn execute<P: LC3>(self, processor: &mut P) {
        match self {
            Self::IAdd(x) => x.execute(processor),
            Self::IAnd(x) => x.execute(processor),
            Self::INot(x) => x.execute(processor),
            Self::IBranch(x) => x.execute(processor),
            Self::IJump(x) => x.execute(processor),
            Self::IJumpSubRoutine(x) => x.execute(processor),
            Self::ILoad(x) => x.execute(processor),
            Self::IStore(x) => x.execute(processor),
            Self::Trap(x) => x.execute(processor),
        }
    }

    fn parse(word: LC3Word) -> Option<Self>
    where
        Self: Sized,
    {
        match get_opcode(word) {
            ADD_OPCODE => Some(Self::IAdd(IAdd::parse(word)?)),
            AND_OPCODE => Some(Self::IAnd(IAnd::parse(word)?)),
            NOT_OPCODE => Some(Self::INot(INot::parse(word)?)),
            BRANCH_OPCODE => Some(Self::IBranch(IBranch::parse(word)?)),
            x if ALL_JUMP_OPCODES.contains(&x) => Some(Self::IJump(IJump::parse(word)?)),
            JSR_OPCODE => Some(Self::IJumpSubRoutine(IJumpSubRoutine::parse(word)?)),
            x if ALL_LOAD_OPCODES.contains(&x) => Some(Self::ILoad(ILoad::parse(word)?)),
            x if ALL_STORE_OPCODES.contains(&x) => Some(Self::IStore(IStore::parse(word)?)),
            TRAP_OPCODE => Some(Self::Trap(Trap::parse(word)?)),
            // Not a known valid opcode
            _ => None,
        }
    }
}

#[cfg(test)]
/// Utility value for calculating instruction range.
///
/// Bottom opcode bit set.
const TWELVE_SET: u16 = 1 << 12;
