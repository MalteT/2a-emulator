//! AST & parser for assembly
//!
//! # Example
//! ```
//! # use emulator_2a_lib::parser::{AsmParser, ParserError, Asm};
//!
//! fn main() {
//!     let asm = r##"#! mrasm
//!
//!         .ORG 0
//!
//!         CLR R0
//!     LOOP:
//!         ST (0xFF), R0
//!         ST (0xF0), R0
//!         INC R0
//!         JR LOOP
//!     "##;
//!
//!     let parsed: Result<Asm, ParserError> = AsmParser::parse(asm);
//!
//!     assert!(parsed.is_ok());
//! }
//! ```

mod ast;
mod parser;

pub use ast::*;
pub use parser::{AsmParser, ParserError};
