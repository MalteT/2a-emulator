//! # Emulator for Minirechner 2a microcomputer
//!
//! ## Usage
//! ```usage
//! 2a-emulator 0.1.0
//! Malte Tammena <malte.tammena@gmx.de>
//! Emulator for the Minirechner 2a microcomputer
//!
//! USAGE:
//!     emulator-2a [FLAGS] [OPTIONS] [--] [PROGRAM]
//!
//! FLAGS:
//!     -c, --check          Validate the given source file
//!     -h, --help           Prints help information
//!     -i, --interactive    Start an interactive session
//!     -V, --version        Prints version information
//!
//! OPTIONS:
//!     -t, --test <TEST>...    Specify a test file
//!
//! ARGS:
//!     <PROGRAM>    File to load and verify
//! ```

use pretty_env_logger;

pub mod compiler;
pub mod error;
pub mod helpers;
pub mod machine;
pub mod tui;

use std::process;

fn main() {
    pretty_env_logger::init();

    match helpers::handle_user_input() {
        Err(e) => {
            println!("{}", e);
            process::exit(1)
        }
        _ => {}
    }
}
