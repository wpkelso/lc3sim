//! Asynchronous [`LC3`] execution.

use std::future::Future;

use crate::executors::LC3;

use super::ExecutionFailure;

/// Progress an LC3 program asynchronously, taking control of memory mapping.
pub trait AsyncHarness {
    type Output: Future<Output = Result<(), ExecutionFailure>>;
    fn step<P: LC3>(&mut self, processor: &mut P) -> Self::Output;
}
