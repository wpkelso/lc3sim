//! Structs to progress [`LC3`] programs.

use thiserror::Error;

use crate::executors::StepFailure;

pub mod r#async;
pub mod general;
pub mod simple;
pub mod sync;

#[derive(Debug, Error)]
pub enum ExecutionFailure {
    #[error(transparent)]
    LC3(#[from] StepFailure),
    #[error("No keyboard is connected, cannot TRAP for input")]
    NoKeyboard,
    #[error("No console is connected, cannot TRAP for text output")]
    NoConsole,
    #[error("No display is connected, cannot write for visual output")]
    NoDisplay,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl PartialEq for ExecutionFailure {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::LC3(x), Self::LC3(y)) => x == y,
            (Self::NoKeyboard, Self::NoKeyboard)
            | (Self::NoConsole, Self::NoConsole)
            | (Self::NoDisplay, Self::NoDisplay) => true,
            _ => false,
        }
    }
}
