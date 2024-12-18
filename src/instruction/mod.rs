//TODO: TRAP instructions

use crate::{defs::LC3Word, executors::LC3};

mod args;
pub use args::*;
mod util;
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
