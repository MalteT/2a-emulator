//! Error module.
//!
//! This module defines the error type used through-out the program.

use emulator_2a_lib::{parser::ParserError, runner::VerificationError};
use pest::error::Error as PestError;
use thiserror::Error;

use std::io::Error as IOError;

use crate::testing::Rule as TestRule;

#[derive(Error, Debug)]
/// THE error type.
pub enum Error {
    /// Thrown when the validation of the ASM source file failes.
    #[error("{_0}")]
    ValidationFailed(#[from] ParserError),
    /// Thrown when, due to IO failure, no ASM source file could be opened.
    #[error("The source file could not be opened!:\n{_0}")]
    OpeningSourceFileFailed(#[from] IOError),
    /// Thrown when a test file failed to parse.
    #[error("{_0}")]
    TestFileParsingError(#[from] PestError<TestRule>),
    /// Thrown when a test failes.
    #[error("Test {_0:?} failed: {_1}")]
    TestFailed(String, String),
    /// Initialization of tui failed.
    #[cfg(feature = "interactive-tui")]
    #[error("Tui initialization failed: {_0}")]
    TuiInitializationFailed(#[source] IOError),
    /// Crossterm backend initialization failed.
    #[cfg(feature = "interactive-tui")]
    #[error("Crossterm initialization failed: {_0}")]
    CrosstermInitializationFailed(#[source] crossterm::ErrorKind),
    /// Crossterm backend exit failed.
    #[cfg(feature = "interactive-tui")]
    #[error("Crossterm exit failed: {_0}")]
    CrosstermExitFailed(#[source] crossterm::ErrorKind),
    /// Verification of a run failed. The first field is an explanation.
    #[error("Verification failed: {_0}")]
    RunVerificationFailed(#[from] VerificationError),
}

impl Error {
    #[cfg(feature = "interactive-tui")]
    pub fn crossterm_init(err: crossterm::ErrorKind) -> Self {
        Error::CrosstermInitializationFailed(err)
    }
    #[cfg(feature = "interactive-tui")]
    pub fn crossterm_exit(err: crossterm::ErrorKind) -> Self {
        Error::CrosstermExitFailed(err)
    }
    #[cfg(feature = "interactive-tui")]
    pub fn tui_init(err: IOError) -> Self {
        Error::TuiInitializationFailed(err)
    }
}
