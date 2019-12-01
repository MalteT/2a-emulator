//! Error module.
//!
//! This module defines the error type used through-out the program.

use failure::Fail;
use parser2a::parser::ParserError;
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
    /// Invalid CLI input.
    InvalidInput(String),
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
            Error::InvalidInput(s) => write!(f, "{}", s),
        }
    }
}
