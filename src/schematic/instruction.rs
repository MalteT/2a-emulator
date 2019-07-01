use bitflags::bitflags;

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
    pub fn reset() -> Self {
        Instruction::OP01
    }
}
