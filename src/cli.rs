use clap::{crate_version, load_yaml, App};
use failure::Fail;
use mr2a_asm_parser::asm::Asm;
use mr2a_asm_parser::parser::{AsmParser, ParserError};

use std::fmt;
use std::fs::read_to_string;
use std::io::Error as IOError;

use crate::compiler::ByteCode;
use crate::tui;

#[derive(Fail, Debug)]
pub enum Error {
    ValidationFailed(#[cause] ParserError),
    OpeningSourceFileFailed(#[cause] IOError),
}

/// Handle user-given CLI parameter.
///
/// This calls the correct parts of the emulator
/// and returns a [`Result`] to be returned to the user.
pub fn handle_user_input() -> Result<(), Error> {
    let yaml = load_yaml!("../static/cli.yml");
    let matches = App::from(yaml).version(crate_version!()).get_matches();

    if matches.is_present("check") {
        validate_source_file(
            matches
                .value_of_lossy("PROGRAM")
                .expect("PROGRAM must be given"),
        )?;
    }
    if matches.is_present("interactive") {
        if matches.is_present("PROGRAM") {
            validate_source_file(matches.value_of_lossy("PROGRAM").expect("Infallible"))?;
        }
        tui::run()?;
    } else if matches.is_present("test") {
        validate_source_file(matches.value_of_lossy("PROGRAM").expect("Infallible"))?;
        println!("Testing functionality is not available yet!");
    } else if !matches.is_present("check") {
        tui::run()?;
    }

    Ok(())
}

/// Validate the given source code file.
/// This fails with an [`Error`] if the source code is not worthy. See [`AsmParser::parse`].
pub fn validate_source_file<P>(path: P) -> Result<(), Error>
where
    P: ToString,
{
    let content = read_to_string(path.to_string())?;
    let program: Asm = AsmParser::parse(&content).map_err(|e| Error::from(e))?;
    let compiled = ByteCode::compile(program);
    println!("{}", compiled);
    Ok(())
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
