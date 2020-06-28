//! # Emulator for Minirechner 2a microcomputer
//!
//! ## Usage
//! ```usage
//! emulator-2a 4.0.0
//! Malte Tammena <malte.tammena@gmx.de>
//! Emulator for the Minirechner 2a microcomputer.
//!
//! If run without arguments an interactive session is started.
//!
//! USAGE:
//!     2a-emulator [SUBCOMMAND]
//!
//! FLAGS:
//!     -h, --help
//!             Prints help information
//!
//!     -V, --version
//!             Prints version information
//!
//!
//! SUBCOMMANDS:
//!     help           Prints this message or the help of the given subcommand(s)
//!     interactive    Run an interactive session
//!     run            Run a single emulation
//!     test           Run tests against a program
//!     verify         Verify the given program's syntax
//! ```

mod args;
mod compiler;
mod error;
mod helpers;
mod machine;
mod runner;
mod supervisor;
mod testing;

#[cfg(feature = "interactive-tui")]
mod tui;

use args::{Args, InteractiveArgs, SubCommand, TestArgs, VerifyArgs};
use error::Error;

use colored::Colorize;
use log::error;

use std::process;

#[paw::main]
fn main(args: Args) {
    pretty_env_logger::init();

    // Match against the given subcommand and execute the part
    // of the program that is requested.
    let result: Result<(), Error> = match args.subcommand {
        Some(SubCommand::Run { .. }) => run_runner(&args),
        Some(SubCommand::Test(args)) => run_tests(&args),
        Some(SubCommand::Verify(args)) => run_verification(&args),
        #[cfg(feature = "interactive-tui")]
        Some(SubCommand::Interactive(args)) => run_interactive_session(&args),
        #[cfg(feature = "interactive-tui")]
        None => run_interactive_session(&InteractiveArgs::default()),
        #[cfg(not(feature = "interactive-tui"))]
        None => {
            println!("Nothing to do..");
            Ok(())
        }
    };

    // Exit with errorcode 1 if an error occured.
    if let Err(e) = result {
        println!("{}: {}", "Error".red().bold(), e);
        process::exit(1)
    }
}

fn run_runner(args: &Args) -> Result<(), Error> {
    error!("Not implemented yet");
    Ok(())
}

fn run_tests(args: &TestArgs) -> Result<(), Error> {
    testing::run_test_with_args(args)
}

fn run_verification(args: &VerifyArgs) -> Result<(), Error> {
    helpers::load_and_verify_source_file(&args.program)
}

#[cfg(feature = "interactive-tui")]
fn run_interactive_session(args: &InteractiveArgs) -> Result<(), Error> {
    // TODO: Move verification here!
    tui::Tui::run_with_args(args)?;
    Ok(())
}
