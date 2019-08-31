//! Types and functions to aid the command line interface.

use clap::{crate_version, load_yaml, App};
use failure::Fail;
use mr2a_asm_parser::asm::Asm;
use mr2a_asm_parser::parser::{AsmParser, ParserError};

use std::fmt;
use std::fs::read_to_string;
use std::io::Error as IOError;

use crate::compiler::Translator;
use crate::tui;

#[derive(Fail, Debug)]
pub enum Error {
    /// Thrown when the validation of the ASM source file failes.
    ValidationFailed(#[cause] ParserError),
    /// Thrown when, due to IO failure, no ASM source file could be opened.
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
        let program = if matches.is_present("PROGRAM") {
            let program = matches.value_of_lossy("PROGRAM").expect("Infallible");
            Some(program)
        } else {
            None
        };
        run_tui(program)?;
    } else if matches.is_present("test") {
        validate_source_file(matches.value_of_lossy("PROGRAM").expect("Infallible"))?;
        println!("Testing functionality is not available yet!");
    } else if !matches.is_present("check") {
        let program = if matches.is_present("PROGRAM") {
            let program = matches.value_of_lossy("PROGRAM").expect("Infallible");
            Some(program)
        } else {
            None
        };
        run_tui(program)?;
    }

    Ok(())
}

/// Run the TUI.
/// If a program was given, run this.
fn run_tui<S: ToString>(program_path: Option<S>) -> Result<(), Error> {
    let program_path = program_path.map(|s| s.to_string());
    let program: Option<Asm> = if let Some(program_path) = program_path {
        let content = read_to_string(program_path.to_string())?;
        Some(AsmParser::parse(&content).map_err(|e| Error::from(e))?)
    } else {
        None
    };
    tui::run(program)?;
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
    println!("{}", program);
    let compiled = Translator::compile(&program);
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
