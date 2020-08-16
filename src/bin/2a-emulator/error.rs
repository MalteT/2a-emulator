//! Error module.
//!
//! This module defines the error type used through-out the program.

use emulator_2a_lib::parser::ParserError;
use failure::Fail;
use pest::error::Error as PestError;

use std::fmt;
use std::io::Error as IOError;

use crate::testing::Rule as TestRule;

#[derive(Fail, Debug)]
/// THE error type.
pub enum Error {
    /// Thrown when the validation of the ASM source file failes.
    ValidationFailed(#[cause] ParserError),
    /// Thrown when, due to IO failure, no ASM source file could be opened.
    OpeningSourceFileFailed(#[cause] IOError),
    /// Thrown when a test file failed to parse.
    TestFileParsingError(#[cause] PestError<TestRule>),
    /// Thrown when a test failes.
    TestFailed(String, String),
    /// Initialization of tui failed.
    #[cfg(feature = "interactive-tui")]
    TuiInitializationFailed(#[cause] IOError),
    /// Crossterm backend initialization failed.
    #[cfg(feature = "interactive-tui")]
    CrosstermInitializationFailed(#[cause] crossterm::ErrorKind),
    /// Crossterm backend exit failed.
    #[cfg(feature = "interactive-tui")]
    CrosstermExitFailed(#[cause] crossterm::ErrorKind),
    /// Verification of a run failed. The first field is an explanation.
    RunVerificationFailed(String),
}

impl From<IOError> for Error {
    fn from(ioe: IOError) -> Self {
        Error::OpeningSourceFileFailed(ioe)
    }
}

impl From<ParserError> for Error {
    fn from(pe: ParserError) -> Self {
        Error::ValidationFailed(pe)
    }
}

impl From<PestError<TestRule>> for Error {
    fn from(pe: PestError<TestRule>) -> Self {
        Error::TestFileParsingError(pe)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::ValidationFailed(pe) => write!(f, "{}", pe),
            Error::OpeningSourceFileFailed(ioe) => {
                write!(f, "The source file could not be opened!:\n{}", ioe)
            }
            Error::TestFileParsingError(pe) => write!(f, "{}", pe),
            Error::TestFailed(n, r) => write!(f, "Test {:?} failed: {}", n, r),
            #[cfg(feature = "interactive-tui")]
            Error::TuiInitializationFailed(ioe) => write!(f, "Tui init failed: {}", ioe),
            #[cfg(feature = "interactive-tui")]
            Error::CrosstermInitializationFailed(cek) => {
                write!(f, "Crossterm init failed: {}", cek)
            }
            #[cfg(feature = "interactive-tui")]
            Error::CrosstermExitFailed(cek) => write!(f, "Crossterm exit failed: {}", cek),
            Error::RunVerificationFailed(reason) => write!(
                f,
                "Verification failed: {:?} did not match expectations",
                reason
            ),
        }
    }
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
