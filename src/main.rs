//! # Emulator for the Minirechner 2a microcomputer
//!
//! This is an emulator for the Minirechner 2a microcomputer. The microcomputer
//! is used during a practical course at the university of Leipzig. This emulator
//! was created for my bachelor thesis and is published under the GNU GPLv3. It's
//! main purpose is to aid students in creating correct solutions for the
//! assignments that are part of the course. it was meant to solve the issue of not
//! having the real hardware at hand while coming up with solutions for the given
//! problems. This FOSS should run on most major plattforms and
//! is versatile enough to test solutions.
//!
//! **Please report all bugs! Just create an issue or message me!**
//!
//! - [Installation](#installation)
//!   - [Prebuild binaries](#prebuild-binaries)
//!   - [The `cargo` way](#the-cargo-way)
//! - [Usage](#usage)
//!   - [`interactive` mode](#interactive-mode)
//!   - [`run`ning programs](#running-programs)
//!   - [`verify`ing programs](#verifying-programs)
//!   - [`test`ing programs](#testing-programs)
//! - [Compilation flags](#compilation-flags)
//!
//!
//! ## Installation
//!
//! ### Prebuild binaries
//!
//! To use one of the prebuild binaries, you can download one of the
//! [release packages](https://github.com/MalteT/2a-emulator/releases).
//! These are always a little outdated, for more up-to-date versions of
//! the emulator download one of the
//! [artifacts created by the CI](https://github.com/MalteT/2a-emulator/actions)!
//!
//! ### The `cargo` way
//!
//! Download and install [rustup](https://rustup.rs/) for your operating
//! system and follow the instructions on how to install
//! [Rust](https://www.rust-lang.org/) using rustup.
//!
//! clone this repository and compile and run your own binary using
//! [Cargo](https://github.com/rust-lang/cargo) (which should
//! have already been installed by rustup):
//! ```console
//! $ cargo run --release
//! ```
//!
//! You may also install the binary using:
//! ```console
//! $ cargo install --git https://github.com/MalteT/2a-emulator
//! ```
//! See the [Cargo Manual](https://doc.rust-lang.org/cargo/commands/cargo-install.html?highlight=install#cargo-install) on where the binary is installed to.
//!
//! ## Usage
//!
//! To simply start an interactive session, run `2a-emulator` in the terminal.
//! If you want to adjust some options before starting the interface, supply them
//! on the command line like this:
//! ```console
//! $ 2a-emulator interactive --ff 42 path_to_a_program.asm
//! ```
//! The above runs an interactive session with `path_to_a_program.asm` compiled
//! and loaded into main memory. Additionally the input register FF contains 42.
//! For a full list of options, see `2a-emulator interactive --help`.
//!
//! You may also be interested in the `run` mode which emulates the execution of a
//! given program for a number of clock cycles before printing the state of the machine.
//! Have a look at `2a-emulator run --help` for more information.
//! To simply verify the syntax of an assembler file run
//! `2a-emulator verify my_faulty_program.asm`.
//!
//! ### `interactive` mode
//!
//! When starting the `2a-emulator` without any parameters or by using `2a-emulator
//! interactive` the interactive terminal user interface (TUI) is started.
//! ```text
//! ┌─┤ Minirechner 2a ├────────────────────┐━╸Info╺━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//! │                                       │Program:
//! │ Outputs:                              │Frequency:                  7.41MHz
//! │ 00000000 00000000                     │Measured Frequency:          0.00Hz
//! │       FF       FE                     │State:                      Running
//! │                                       │━╸Program╺━━━━━━━━━━━━━━━━━━━━━━━━━
//! │ Inputs:                               │
//! │ 00000000 00000000 00000000 00000000   │
//! │       FF       FE       FD       FC   │
//! │                                       │
//! │ Registers:                            │
//! │ R0 00000000                           │
//! │ R1 00000000                           │
//! │ R2 00000000                           │
//! │ PC 00000000                           │━╸Keybindings╺━━━━━━━━━━━━━━━━━━━━━
//! │ FR 00000000                           │Clock                         Enter
//! │ SP 00000000                           │Toggle autorun               CTRL+A
//! │ R6 00000000                           │Toggle asm step              CTRL+W
//! │ R7 00000000                           │Reset                        CTRL+R
//! │                                       │Edge interrupt               CTRL+E
//! │                                       │Continue                     CTRL+L
//! │                                       │━╸Commands╺━━━━━━━━━━━━━━━━━━━━━━━━
//! │                                       │load PATH          Load asm program
//! │                                   ... │set …             Change a settings
//! │                                       │unset …        Unset a bool setting
//! │───────────────────────────────────────│show …       Select part to display
//! │> █                                    │quit               Exit the program
//! └───────────────────────────────────────┘───────────────────────────────────
//! ```
//!
//! Let's annotate the interface to make sure that we're all on the same page.
//!
//! ```text
//! ┌─┤ Minirechner 2a ├────────────────────┐━╸Info╺━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//! │                                       │
//! │ Outputs:                              │ General information about the
//! │ 00000000 00000000  <-In- and Output   │ running program/machine.
//! │       FF       FE    registers        │
//! │                      |                │━╸Program╺━━━━━━━━━━━━━━━━━━━━━━━━━
//! │ Inputs:              V                │
//! │ 00000000 00000000 00000000 00000000   │ If you've loaded a program, the
//! │       FF       FE       FD       FC   │ assembler will appear here. A
//! │                                       │ marker shows what's currently
//! │ Registers:                            │ being executed, to visualize
//! │ R0 00000000                           │ the control flow.
//! │ R1 00000000                           │
//! │ R2 00000000                           │
//! │ PC 00000000                           │━╸Keybindings╺━━━━━━━━━━━━━━━━━━━━━
//! │ FR 00000000                           │
//! │ SP 00000000                           │ This is a list of keybindings.
//! │ R6 00000000                           │ Gray means, hitting that has
//! │ R7 00000000                           │ no effect.
//! │                                       │
//! │   ^                                   │
//! │   |                                   │━╸Commands╺━━━━━━━━━━━━━━━━━━━━━━━━
//! │   Register block containing           │
//! │   eight registers                 ... │ This is a list of possible
//! │                                       │ completions for the input field.
//! │───────────────────────────────────────│
//! │> This is the input field              │
//! └───────────────────────────────────────┘───────────────────────────────────
//! ```
//!
//! **See `2a-emulator interactive --help` for a full list of options.**
//!
//! ### `run`ning programs
//!
//! Using `2a-emulator run` it is possible to run assembler programs without
//! interaction. **This is still in development!**
//!
//! Consider the following example:
//!
//! ```console
//! $ 2a-emulator run \
//!         programs/11-simple-addition.asm \
//!         100 \
//!         --fc 10 \
//!         --fd 42
//! ```
//!
//! The first parameter is the path to the program we want to execute, the second
//! the number of clock cycles to emulate at most. These are the only two required
//! parameters. Now, `programs/11-simple-addition.asm` takes the inputs from FC/FD
//! and adds them. The result is written to output register FF. To make this example
//! worthwhile we added `--fc` and `--fd` to the argument list and supplied values
//! which will be written to the input registers FC/FD respectively.
//!
//! This may result in the following output:
//!
//! ```text
//! Program: programs/11-simple-addition.asm
//! Time:    10us 35ns
//! Cycles:  100/100
//! State:   Running
//! Output:  FE: 0
//!          FF: 52
//! ```
//!
//! As we can see, `52` has indeed been written to output register FF, while output
//! register FE was either untouched or containes a zero. Additionally some information
//! about the `run` is shown, most importantly the number of cycles that were executed
//! and the state of the machine after executing these cycles. In our example the machine
//! is still `running`, alternatives are `stopped` and `error`.
//!
//! **See `2a-emulator run --help` for a full list of options.**
//!
//! ### `verify`ing programs
//!
//! Basic functionality exists to verify programs. As of yet only three
//! machine parameters can be tested against and it works like this. Consider the
//! above example, now extended like this:
//!
//! ```console
//! $ 2a-emulator run \
//!         programs/11-simple-addition.asm \
//!         100 \
//!         --fc 10 \
//!         --fd 42 \
//!         verify \
//!         --ff 52 \
//!         --state running
//! ```
//!
//! Note the last three lines which weren't there before. The `verify` begins the
//! verification arguments. The first thing we expect from the simple addition program
//! is the correct sum in output register FF. The second expectation includes the
//! state after execution. The machine should neither stop nor throw an error.
//! The output after executing the above command is the same as without the verify suffix,
//! but only because all expectation were met.
//! If we would've written the following instead:
//!
//! ```console
//! $ 2a-emulator run \
//!         programs/11-simple-addition.asm \
//!         100 \
//!         --fc 10 \
//!         --fd 42 \
//!         verify \
//!         --ff 51 \
//!         --state running
//! ```
//!
//! The result would include the following line of output:
//!
//! ```text
//! Error: Verification failed: "Output register FF" did not match expectations
//! ```
//!
//! Additionally the exit code of the program is non-zero, which marks that something
//! failed. This can be used to build more complex verifications using shell scripts.
//!
//! **See `2a-emulator run verify --help` for a full list of options.**
//!
//! ### `test`ing programs
//!
//! There is a `test` subcommand available at the moment, which will hopefully be
//! replaced by the `verify` subcommand soon. But if you want to learn more about
//! it, have a look at `2a-emulator test --help` and go through the test examples
//! in `./program_tests`.
//!
//! ## Compilation flags
//!
//! The following feature flags can be used to influence the generated binary.
//! *See [here](https://doc.rust-lang.org/cargo/reference/features.html)*.
//!
//! - `interactive-tui` (*opt-out*) enables the interactive session.
//!   Without it, no interactive session is possible.
//! - `utf8` (*opt-in*) enables the use of character codes which are supported
//!   by fewer terminals. Note, that at the moment the difference is marginal.
//!
//!   **Warning**: The Ubuntu Terminal in combination with the Ubuntu Mono font has
//!   troubles displaying some characters. Thus the `utf8` feature
//!   should not be used on machines running Ubuntu.

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

use args::{Args, InteractiveArgs, RunArgs, SubCommand, TestArgs, VerifyArgs};
use error::Error;

use colored::Colorize;

use std::process;

#[paw::main]
fn main(args: Args) {
    pretty_env_logger::init();

    // Match against the given subcommand and execute the part
    // of the program that is requested.
    let result: Result<(), Error> = match args.subcommand {
        Some(SubCommand::Run(args)) => run_runner(&args),
        Some(SubCommand::Test(args)) => run_tests(&args),
        Some(SubCommand::Verify(args)) => run_verification(&args),
        #[cfg(feature = "interactive-tui")]
        Some(SubCommand::Interactive(args)) => run_interactive_session(&args),
        #[cfg(feature = "interactive-tui")]
        None => run_interactive_session(&InteractiveArgs::default()),
        #[cfg(not(feature = "interactive-tui"))]
        None => {
            eprintln!("Nothing to do..");
            Ok(())
        }
    };

    // Exit with errorcode 1 if an error occured.
    if let Err(e) = result {
        eprintln!("{}: {}", "Error".red().bold(), e);
        process::exit(1)
    }
}

fn run_runner(args: &RunArgs) -> Result<(), Error> {
    let run = runner::Runner::with_args(args)?;
    run.run()
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
