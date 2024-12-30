//! Dangerous and/or inefficient LC3 executors.

use std::future::{ready, Ready};

use crate::{
    executors::LC3,
    instruction::{InstructionEnum, Trap},
};

use super::{r#async::AsyncHarness, sync::SyncHarness, ExecutionFailure};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Treats I/O as disconnected, causing infinite stalls on any I/O TRAP.
pub struct IgnoreIO;

impl SyncHarness for IgnoreIO {
    fn step<P: LC3>(&mut self, processor: &mut P) -> Result<(), ExecutionFailure> {
        println!(
            "PC, Inst: {:#x}, {:?}",
            processor.pc(),
            processor.cur_inst().unwrap()
        );
        Ok(processor.step()?)
    }
}

impl AsyncHarness for IgnoreIO {
    type Output = Ready<Result<(), ExecutionFailure>>;
    fn step<P: LC3>(&mut self, processor: &mut P) -> Self::Output {
        ready(<Self as SyncHarness>::step(self, processor))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Errors on any I/O TRAP.
pub struct FailIO;

impl SyncHarness for FailIO {
    fn step<P: LC3>(&mut self, processor: &mut P) -> Result<(), ExecutionFailure> {
        match processor.cur_inst() {
            Some(InstructionEnum::Trap(Trap::Getc)) => return Err(ExecutionFailure::NoKeyboard),
            Some(InstructionEnum::Trap(Trap::Out)) => return Err(ExecutionFailure::NoConsole),
            Some(InstructionEnum::Trap(Trap::PutS)) => return Err(ExecutionFailure::NoConsole),
            Some(InstructionEnum::Trap(Trap::In)) => return Err(ExecutionFailure::NoConsole),
            Some(InstructionEnum::Trap(Trap::PutSp)) => return Err(ExecutionFailure::NoConsole),
            _ => (),
        }

        Ok(processor.step()?)
    }
}

impl AsyncHarness for FailIO {
    type Output = Ready<Result<(), ExecutionFailure>>;
    fn step<P: LC3>(&mut self, processor: &mut P) -> Self::Output {
        ready(<Self as SyncHarness>::step(self, processor))
    }
}
