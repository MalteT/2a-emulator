//! Library for the 2a-emulator

//mod compiler;
//mod error;
//mod helpers;
mod machine;
//mod supervisor;
mod alu;
mod microprogram_ram;
mod register;
mod signal;

pub use alu::{AluInput, AluOutput, AluSelect};
pub use machine::{Machine, State, Board, DAISR, DAICR};
pub use microprogram_ram::{MicroprogramRam, Word};
pub use register::{Flags, Register, RegisterNumber};
pub use signal::Signal;
