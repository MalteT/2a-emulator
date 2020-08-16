//! # Minirechner2a ASM parser
//!
//! # Example
//! ```
//! # use emulator_2a_lib::parser::AsmParser;
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
//!     let parsed = AsmParser::parse(asm);
//!
//!     match parsed {
//!         Ok(parsed) => {
//!             #[cfg(feature = "formatting")]
//!             println!("{}", parsed);
//!             #[cfg(not(feature = "formatting"))]
//!             println!("{:?}", parsed);
//!         },
//!         Err(e) => panic!("Whoooops {}", e),
//!     }
//! }
//! ```

pub mod ast;
pub mod parser;
