use super::{Instruction, MP28BitWord};

/// The register block.
/// Containing `R0` through `R7`
#[derive(Debug, Clone)]
pub struct Register {
    content: [u8; 8],
}

impl Register {
    /// Create a new register block.
    pub fn new() -> Self {
        let content = [0; 8];
        Register { content }
    }
    /// Get current data output A of the register.
    pub fn doa(&self, inst: &Instruction, word: &MP28BitWord) -> u8 {
        let (a2, a1, a0) = if word.contains(MP28BitWord::MRGAA3) {
            (
                false,
                inst.contains(Instruction::OP01),
                inst.contains(Instruction::OP00),
            )
        } else {
            (
                word.contains(MP28BitWord::MRGAA2),
                word.contains(MP28BitWord::MRGAA1),
                word.contains(MP28BitWord::MRGAA0),
            )
        };
        let addr = (a2 as usize) << 2 + (a1 as usize) << 1 + a0 as usize;
        self.content[addr]
    }
    /// Get current data output B of the register.
    pub fn dob(&self, inst: &Instruction, word: &MP28BitWord) -> u8 {
        let (b2, b1, b0) = if word.contains(MP28BitWord::MRGAB3) {
            (
                false,
                inst.contains(Instruction::OP11),
                inst.contains(Instruction::OP10),
            )
        } else {
            (
                word.contains(MP28BitWord::MRGAB2),
                word.contains(MP28BitWord::MRGAB1),
                word.contains(MP28BitWord::MRGAB0),
            )
        };
        let addr = (b2 as usize) << 2 + (b1 as usize) << 1 + b0 as usize;
        self.content[addr]
    }
}
