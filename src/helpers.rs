//! Types and Functions to aid the program.

use ::tui::style::Color;
use ::tui::style::Modifier;
use ::tui::style::Style;
use clap::{crate_version, load_yaml, App};
use lazy_static::lazy_static;
use log::trace;
use parser2a::asm::Asm;
use parser2a::parser::AsmParser;

use std::fs::read_to_string;
use std::path::PathBuf;

use crate::compiler::Translator;
use crate::error::Error;
use crate::testing::TestFile;
use crate::tui;

lazy_static! {
    pub static ref DIMMED: Style = Style::default().modifier(Modifier::DIM);
    pub static ref YELLOW: Style = Style::default().fg(Color::Yellow);
    pub static ref RED: Style = Style::default().fg(Color::Red);
    pub static ref GREEN: Style = Style::default().fg(Color::Green);
}

/// Handle user-given CLI parameter.
///
/// This calls the correct parts of the emulator
/// and returns a [`Result`] to be returned to the user.
pub fn handle_user_input() -> Result<(), Error> {
    let yaml = load_yaml!("../static/cli.yml");
    let matches = App::from(yaml).version(crate_version!()).get_matches();

    if matches.is_present("check") {
        cli_validate_source_file(
            matches
                .value_of_lossy("PROGRAM")
                .expect("PROGRAM must be given")
                .to_string(),
        )?;
    }
    if matches.is_present("interactive") {
        let program_path = if matches.is_present("PROGRAM") {
            let program_path = matches
                .value_of_lossy("PROGRAM")
                .expect("Infallible")
                .to_string();
            Some(program_path)
        } else {
            None
        };
        run_tui(program_path)?;
    } else if matches.is_present("test") {
        let tests = matches.values_of_lossy("test").expect("TEST must be given");
        let program = matches
            .value_of_lossy("PROGRAM")
            .expect("Unfallible")
            .to_string();
        for test_path in tests {
            execute_test(test_path, &program)?
        }
    } else if !matches.is_present("check") {
        let program = if matches.is_present("PROGRAM") {
            let program = matches
                .value_of_lossy("PROGRAM")
                .expect("Infallible")
                .to_string();
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
fn run_tui<P: Into<PathBuf>>(program_path: Option<P>) -> Result<(), Error> {
    let tui = tui::Tui::new()?;
    tui.run(program_path)?;
    Ok(())
}

/// Execute a test given by it's path.
fn execute_test<P1, P2>(test_path: P1, program_path: P2) -> Result<(), Error>
where
    P1: Into<PathBuf>,
    P2: Into<PathBuf>,
{
    let test_path: PathBuf = test_path.into();
    let program_path: PathBuf = program_path.into();
    trace!("Executing tests from file {:?}", test_path);
    TestFile::parse(&test_path)?.execute_against(&program_path)?;
    println!(
        "Tests in {:?} ran successful against {:?}!",
        test_path, program_path
    );
    Ok(())
}

/// Validate the given source code file.
/// This fails with an [`Error`] if the source code is not worthy. See [`AsmParser::parse`].
pub fn cli_validate_source_file<P>(path: P) -> Result<(), Error>
where
    P: Into<PathBuf>,
{
    let program = read_asm_file(path)?;
    println!("{}", program);
    let compiled = Translator::compile(&program);
    println!("{}", compiled);
    Ok(())
}

/// Read the given path to valid [`Asm`] or fail.
pub fn read_asm_file<P>(path: P) -> Result<Asm, Error>
where
    P: Into<PathBuf>,
{
    let content = read_to_string(path.into())?;
    Ok(AsmParser::parse(&content).map_err(|e| Error::from(e))?)
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
