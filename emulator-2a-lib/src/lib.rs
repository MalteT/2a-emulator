// XXX: proptest causes issues with clippy
#![allow(clippy::unit_arg)]
//! Library for the 2a-emulator

//mod error;
//mod helpers;
pub mod compiler;
pub mod machine;
pub mod parser;
pub mod runner;
