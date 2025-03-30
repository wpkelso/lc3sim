//! Generically defined executors

use std::future::{ready, Ready};

use crate::{
    executors::LC3,
    instruction::{InstructionEnum, Trap},
};

use super::{r#async::AsyncHarness, sync::SyncHarness, ExecutionFailure};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Implements I/O over any well defined Rust I/O
pub struct GeneralIO<W, R> {
    writer: W,
    reader: R,
}

impl<W, R> SyncHarness for GeneralIO<W, R>
where
    W: std::io::Write,
    R: std::io::Read,
{
    fn step<P: LC3>(&mut self, processor: &mut P) -> Result<(), ExecutionFailure> {
        // Write out any pending characters
        if let Some(crt) = processor.pop_crt() {
            self.writer.write_all(&[crt])?;
        }

        // Write out any pending characters
        if let Some(crt) = processor.pop_crt() {
            self.writer.write_all(&[crt])?;
        }

        Ok(processor.step()?)
    }
}

/*
impl<W, R> AsyncHarness for GeneralIO<W, R> {
    type Output = Ready<Result<(), ExecutionFailure>>;
    fn step<P: LC3>(&mut self, processor: &mut P) -> Self::Output {
        ready(<Self as SyncHarness>::step(self, processor))
    }
}
*/
