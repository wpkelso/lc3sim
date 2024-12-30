//! Synchronous [`LC3`] execution.

use crate::executors::{StepFailure, LC3};

use super::ExecutionFailure;

/// Progress an LC3 program synchronously, taking control of memory mapping.
pub trait SyncHarness {
    fn step<P: LC3>(&mut self, processor: &mut P) -> Result<(), ExecutionFailure>;
}

/// Runs `processor's` program to completion on `harness`.
pub fn step_continue<H: SyncHarness, P: LC3>(
    harness: &mut H,
    processor: &mut P,
) -> Result<(), ExecutionFailure> {
    loop {
        if let Err(e) = harness.step(processor) {
            if e == ExecutionFailure::LC3(StepFailure::Halted) {
                return Ok(());
            } else {
                return Err(e)?;
            }
        }
    }
}

/// Limited run of `processor's` program to completion on `harness`.
///
/// Makes at most `limit` steps. Returns true if program ran to completition,
/// returns false if the program reached its limit.
pub fn lim_step_continue<H: SyncHarness, P: LC3>(
    harness: &mut H,
    processor: &mut P,
    limit: u64,
) -> Result<bool, ExecutionFailure> {
    for _ in 0..limit {
        if let Err(e) = harness.step(processor) {
            if e == ExecutionFailure::LC3(StepFailure::Halted) {
                return Ok(true);
            } else {
                return Err(e);
            }
        }
    }

    // Reached the step limit
    Ok(false)
}
