use failure::Fail;
use mr2a_asm_parser::parser::{ParserError};

use std::io::Error as IOError;
use std::fmt;

#[derive(Fail, Debug)]
pub enum Error {
    /// Thrown when the validation of the ASM source file failes.
    ValidationFailed(#[cause] ParserError),
    /// Thrown when, due to IO failure, no ASM source file could be opened.
    OpeningSourceFileFailed(#[cause] IOError),
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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::ValidationFailed(pe) => write!(f, "{}", pe),
            Error::OpeningSourceFileFailed(ioe) => {
                write!(f, "The source file could not be opened!:\n{}", ioe)
            }
        }
    }
}
