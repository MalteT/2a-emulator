use bitflags::bitflags;

/// The instruction register.
///
/// It stores the currently executed [`Instruction`].
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct InstructionRegister {
    content: Instruction,
}

impl InstructionRegister {
    /// Create a new register.
    ///
    /// **Note**: The default instruction is not actually just zeros, but contains
    /// [`Instruction::OP01`].
    ///
    /// # Example
    ///
    /// ```
    /// # use emulator_2a_lib::{InstructionRegister, Instruction};
    /// let ir = InstructionRegister::new();
    ///
    /// assert_eq!(*ir.get(), Instruction::OP01);
    /// ```
    pub const fn new() -> Self {
        InstructionRegister {
            content: Instruction::reset(),
        }
    }
    /// Get a reference to the content of this register.
    pub const fn get(&self) -> &Instruction {
        &self.content
    }
    /// Get the raw 8-bit value that is contained in the register.
    pub const fn get_raw(&self) -> u8 {
        self.content.bits()
    }
    /// Update the content of the register with the supplied [`Instruction`].
    pub fn set(&mut self, instruction: Instruction) {
        self.content = instruction;
    }
    /// Update the content of the register with the supplied raw [`Instruction`].
    pub fn set_raw(&mut self, instruction: u8) {
        self.content = Instruction::from_bits_truncate(instruction);
    }
    /// Reset the register to the default state. See [`Instruction::reset`].
    pub fn reset(&mut self) {
        self.content = Instruction::reset();
    }
}

bitflags! {
    /// A single byte handled by the instruction register.
    pub struct Instruction: u8 {
        const A8   = 0b10000000;
        const A7   = 0b01000000;
        const A6   = 0b00100000;
        const A5   = 0b00010000;
        const OP11 = 0b00001000;
        const OP10 = 0b00000100;
        const OP01 = 0b00000010;
        const OP00 = 0b00000001;
    }
}

impl Instruction {
    /// Create the default instruction, that is used by the Minirechner 2a,
    /// whenever a reset is received: `0x02`
    // TODO: Why this value?
    pub const fn reset() -> Self {
        Instruction::OP01
    }
}
