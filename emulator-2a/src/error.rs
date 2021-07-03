//! Error module.
//!
//! This module defines the error type used through-out the program.

use emulator_2a_lib::{parser::ParserError, runner::VerificationError};
use thiserror::Error;

use std::io::Error as IOError;

#[derive(Error, Debug)]
/// THE error type.
pub enum Error {
    /// Thrown when the validation of the ASM source file failes.
    #[error("{_0}")]
    Validation(#[from] ParserError),
    /// Thrown when, due to IO failure, no ASM source file could be opened.
    #[error("The source file could not be opened!:\n{_0}")]
    OpeningSourceFile(#[from] IOError),
    /// Initialization of tui failed.
    #[cfg(feature = "interactive-tui")]
    #[error("Tui initialization failed: {_0}")]
    TuiInitialization(#[source] IOError),
    /// Crossterm backend initialization failed.
    #[cfg(feature = "interactive-tui")]
    #[error("Crossterm initialization failed: {_0}")]
    CrosstermInitialization(#[source] crossterm::ErrorKind),
    /// Crossterm backend exit failed.
    #[cfg(feature = "interactive-tui")]
    #[error("Crossterm exit failed: {_0}")]
    CrosstermExit(#[source] crossterm::ErrorKind),
    /// Verification of a run failed. The first field is an explanation.
    #[error("Verification failed: {_0}")]
    RunVerification(#[from] VerificationError),
}

impl Error {
    #[cfg(feature = "interactive-tui")]
    pub fn crossterm_init(err: crossterm::ErrorKind) -> Self {
        Error::CrosstermInitialization(err)
    }
    #[cfg(feature = "interactive-tui")]
    pub fn crossterm_exit(err: crossterm::ErrorKind) -> Self {
        Error::CrosstermExit(err)
    }
    #[cfg(feature = "interactive-tui")]
    pub fn tui_init(err: IOError) -> Self {
        Error::TuiInitialization(err)
    }
}
