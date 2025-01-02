//! Structs to progress [`LC3`] programs.

use thiserror::Error;

use crate::executors::StepFailure;

pub mod r#async;
pub mod simple;
pub mod sync;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Error)]
pub enum ExecutionFailure {
    #[error(transparent)]
    LC3(#[from] StepFailure),
    #[error("No keyboard is connected, cannot TRAP for input")]
    NoKeyboard,
    #[error("No console is connected, cannot TRAP for text output")]
    NoConsole,
    #[error("No display is connected, cannot write for visual output")]
    NoDisplay,
}
