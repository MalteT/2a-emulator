//! Types and Functions to aid the program.

use colored::Colorize;
use parser2a::asm::Asm;
use parser2a::parser::AsmParser;

use std::fs::read_to_string;
use std::path::PathBuf;

use crate::error::Error;

#[cfg(feature = "interactive-tui")]
mod constants;

#[cfg(feature = "interactive-tui")]
pub use constants::*;

/// Loads and verifies the source file found at `path`.
/// This fails with an [`Error`] if the source code is not worthy.
/// See [`AsmParser::parse`].
pub fn load_and_verify_source_file<P>(path: P) -> Result<(), Error>
where
    P: Into<PathBuf>,
{
    let path: PathBuf = path.into();
    read_asm_file(&path)?;
    println!(
        "Source file {} is valid.",
        path.to_string_lossy().bright_green()
    );
    Ok(())
}

/// Read the given path to valid [`Asm`] or fail.
pub fn read_asm_file<P>(path: P) -> Result<Asm, Error>
where
    P: Into<PathBuf>,
{
    let content = read_to_string(path.into())?;
    Ok(AsmParser::parse(&content).map_err(Error::from)?)
}

/// Format a number using the suffixes `k`, `M`, `G` when useful.
pub fn format_number(mut nr: f32) -> String {
    let mut suffix = "";
    if nr > 2_000_000_000.0 {
        nr /= 1_000_000_000.0;
        suffix = "G"
    } else if nr > 2_000_000.0 {
        nr /= 1_000_000.0;
        suffix = "M"
    } else if nr > 2_000.0 {
        nr /= 1_000.0;
        suffix = "k"
    }
    format!("{:.2}{}Hz", nr, suffix)
}
