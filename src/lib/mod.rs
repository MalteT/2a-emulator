//! Library for the 2a-emulator

//mod compiler;
//mod error;
//mod helpers;
mod alu;
mod board;
mod bus;
mod instruction;
mod interface;
mod machine;
mod microprogram_ram;
mod register;
mod signal;

pub use alu::{AluInput, AluOutput, AluSelect};
pub use board::{Board, DAICR, DAISR, DASR};
pub use bus::Bus;
pub use instruction::{Instruction, InstructionRegister};
pub use interface::MachineInterface;
pub use machine::{Machine, State};
pub use microprogram_ram::{MicroprogramRam, Word};
pub use register::{Flags, Register, RegisterNumber};
pub use signal::Signal;

pub(crate) use machine::Interrupt;
