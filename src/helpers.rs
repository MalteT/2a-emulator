//! Types and Functions to aid the program.

use clap::{crate_version, load_yaml, App, ArgMatches};
use log::trace;
use parser2a::asm::Asm;
use parser2a::parser::AsmParser;

use std::fs::read_to_string;
use std::path::PathBuf;

use crate::error::Error;
use crate::testing::TestFile;

#[cfg(feature = "interactive-tui")]
mod constants;

#[cfg(feature = "interactive-tui")]
pub use constants::*;

/// Initial configuration for the machine, given through the CLI.
#[derive(Debug, Clone)]
pub struct Configuration {
    /// 8-bit input port.
    pub irg: u8,
    /// Temperature sensor. \[0.0 - 2.55 \]
    pub temp: f32,
    /// Jumper J1 and J2.
    pub jumper: [bool; 2],
    /// Analog inputs I1 and I2.
    pub analog_inputs: [f32; 2],
    /// Universal I/O pins UIO1, UIO2 and UIO3.
    pub uios: [Option<bool>; 3],
}

impl Configuration {
    fn from_matches(matches: &ArgMatches) -> Result<Self, Error> {
        let err_int =
            |e, s| Error::InvalidInput(format!("Could not parse integer value of {}: {}", e, s));
        let err_float =
            |e, s| Error::InvalidInput(format!("Could not parse float value of {}: {}", e, s));
        let err_bool =
            |e, s| Error::InvalidInput(format!("Could not parse bool value of {}: {}", e, s));
        let parse_uio = |v: &str, s| v.parse().map_err(|e| err_bool(e, s));
        let irg = matches
            .value_of_lossy("irg")
            .expect("IRG has default")
            .parse()
            .map_err(|e| err_int(e, "IRG"))?;
        let jumper = [matches.is_present("j1"), !matches.is_present("no-j2")];
        let i1 = matches
            .value_of_lossy("i1")
            .expect("I1 has default")
            .parse()
            .map_err(|e| err_float(e, "I1"))?;
        let i2 = matches
            .value_of_lossy("i2")
            .expect("I2 has default")
            .parse()
            .map_err(|e| err_float(e, "I2"))?;
        let analog_inputs = [i1, i2];
        let uio1 = matches
            .value_of_lossy("uio1")
            .map(|v| parse_uio(&v, "UIO1"))
            .transpose()?;
        let uio2 = matches
            .value_of_lossy("uio2")
            .map(|v| parse_uio(&v, "UIO2"))
            .transpose()?;
        let uio3 = matches
            .value_of_lossy("uio3")
            .map(|v| parse_uio(&v, "UIO3"))
            .transpose()?;
        let temp = matches
            .value_of_lossy("temp")
            .expect("TEMP has default")
            .parse()
            .map_err(|e| err_float(e, "TEMP"))?;
        let uios = [uio1, uio2, uio3];
        Ok(Configuration {
            jumper,
            irg,
            analog_inputs,
            temp,
            uios,
        })
    }
}

/// Handle user-given CLI parameter.
///
/// This calls the correct parts of the emulator
/// and returns a [`Result`] to be returned to the user.
pub fn handle_user_input() -> Result<(), Error> {
    let yaml = load_yaml!("../static/cli.yml");
    let matches = App::from(yaml).version(crate_version!()).get_matches();

    // Parse configuration
    let conf = Configuration::from_matches(&matches)?;

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
        run_tui(program_path, &conf)?;
    } else if matches.is_present("test") {
        let tests = matches.values_of_lossy("test").expect("TEST must be given");
        let program = matches
            .value_of_lossy("PROGRAM")
            .expect("Unfallible")
            .to_string();
        for test_path in tests {
            execute_test(test_path, &program, &conf)?
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
        run_tui(program, &conf)?;
    }

    Ok(())
}

/// Run the TUI.
/// If a program was given, run this.
#[cfg(feature = "interactive-tui")]
fn run_tui<P: Into<PathBuf>>(program_path: Option<P>, conf: &Configuration) -> Result<(), Error> {
    use crate::tui::Tui;
    let tui = Tui::new(conf)?;
    tui.run(program_path)?;
    Ok(())
}

/// Dummy run_tui
/// This just logs that the feature is not enabled.
#[cfg(not(feature = "interactive-tui"))]
fn run_tui<P: Into<PathBuf>>(program_path: Option<P>, conf: &Configuration) -> Result<(), Error> {
    ::log::warn!("Compiled without the 'interactive-tui' feature!");
    Ok(())
}

/// Execute a test given by it's path.
fn execute_test<P1, P2>(test_path: P1, program_path: P2, conf: &Configuration) -> Result<(), Error>
where
    P1: Into<PathBuf>,
    P2: Into<PathBuf>,
{
    let test_path: PathBuf = test_path.into();
    let program_path: PathBuf = program_path.into();
    trace!("Executing tests from file {:?}", test_path);
    TestFile::parse(&test_path)?.execute_against(&program_path, conf)?;
    println!(
        "Tests in {:?} ran successful against {:?}!",
        test_path, program_path
    );
    Ok(())
}

/// Validate the given source code file.
/// This fails with an [`Error`] if the source code is not worthy.
/// See [`AsmParser::parse`].
pub fn cli_validate_source_file<P>(path: P) -> Result<(), Error>
where
    P: Into<PathBuf>,
{
    let path: PathBuf = path.into();
    read_asm_file(&path)?;
    println!("Source file {:?} is valid.", path);
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
