use parser2a::asm::Stacksize;

use super::Signal;

/// The register block.
/// Containing `R0` through `R7`
#[derive(Debug, Clone)]
pub struct Register {
    pub(crate) content: [u8; 8],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// All possible register.
pub enum RegisterNumber {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
}

impl Register {
    /// Create a new register block.
    pub fn new() -> Self {
        let content = [0; 8];
        Register { content }
    }
    /// Get current data output A of the register.
    pub fn doa(&self, signal: &Signal) -> u8 {
        let (a2, a1, a0) = if signal.mrgaa3() {
            (false, signal.op01(), signal.op00())
        } else {
            (signal.mrgaa2(), signal.mrgaa1(), signal.mrgaa0())
        };
        let addr = ((a2 as usize) << 2) + ((a1 as usize) << 1) + (a0 as usize);
        self.content[addr]
    }
    /// Get current data output B of the register.
    pub fn dob(&self, signal: &Signal) -> u8 {
        let (b2, b1, b0) = if signal.mrgab3() {
            (false, signal.op11(), signal.op10())
        } else {
            (signal.mrgab2(), signal.mrgab1(), signal.mrgab0())
        };
        let addr = ((b2 as usize) << 2) + ((b1 as usize) << 1) + (b0 as usize);
        self.content[addr]
    }
    /// Clear the register block.
    pub fn reset(&mut self) {
        self.content = [0; 8];
    }
    /// Derive the selected register from the given [`Signal`]s.
    pub fn get_selected(signal: &Signal) -> RegisterNumber {
        let (a2, a1, a0) = if signal.mrgws() {
            // Write to address selected by b
            if signal.mrgab3() {
                (false, signal.op11(), signal.op10())
            } else {
                (signal.mrgab2(), signal.mrgab1(), signal.mrgab0())
            }
        } else {
            // Write to address selected by a
            if signal.mrgaa3() {
                (false, signal.op01(), signal.op00())
            } else {
                (signal.mrgaa2(), signal.mrgaa1(), signal.mrgaa0())
            }
        };
        match (a2, a1, a0) {
            (false, false, false) => RegisterNumber::R0,
            (false, false, true) => RegisterNumber::R1,
            (false, true, false) => RegisterNumber::R2,
            (false, true, true) => RegisterNumber::R3,
            (true, false, false) => RegisterNumber::R4,
            (true, false, true) => RegisterNumber::R5,
            (true, true, false) => RegisterNumber::R6,
            (true, true, true) => RegisterNumber::R7,
        }
    }
    /// Write a new value into the register.
    /// The register number will be derived from the given signals
    pub fn write(&mut self, signal: &Signal, value: u8) {
        let selected: usize = Register::get_selected(signal).into();
        self.content[selected] = value;
    }
    /// Update flags in R4.
    pub fn write_flags(&mut self, signal: &Signal) {
        // Persistent IEF
        let mut value = (self.ief() as u8) << 3;
        if signal.co() {
            value |= 0b0000_0001;
        }
        if signal.zo() {
            value |= 0b0000_0010;
        }
        if signal.no() {
            value |= 0b0000_0100;
        }
        self.content[4] = value;
    }
    /// Update the interrupt enable flag.
    pub fn set_ief(&mut self) {
        self.content[4] |= 0b0000_1000;
    }
    /// Get current carry flag.
    pub fn cf(&self) -> bool {
        self.content[4] & 0b0001 != 0
    }
    /// Get current zero flag.
    pub fn zf(&self) -> bool {
        self.content[4] & 0b0010 != 0
    }
    /// Get current negative flag.
    pub fn nf(&self) -> bool {
        self.content[4] & 0b0100 != 0
    }
    /// Get current interrupt enable flag.
    pub fn ief(&self) -> bool {
        self.content[4] & 0b1000 != 0
    }
    /// Write the given `value` to the given [`RegisterNumber`].
    pub fn set(&mut self, reg: RegisterNumber, value: u8) {
        let reg: usize = reg.into();
        self.content[reg] = value;
    }
    /// Update the CO flag.
    pub fn update_co(&mut self, co: bool) {
        let co = (co as u8) << 2;
        self.content[4] = (self.content[4] & 0b1111_1011) | co;
    }
    /// Update the ZO flag.
    pub fn update_zo(&mut self, zo: bool) {
        let zo = (zo as u8) << 1;
        self.content[4] = (self.content[4] & 0b1111_1101) | zo;
    }
    /// Update the NO flag.
    pub fn update_no(&mut self, no: bool) {
        let no = no as u8;
        self.content[4] = (self.content[4] & 0b1111_1110) | no;
    }
    /// Get register content.
    pub fn get(&self, rn: RegisterNumber) -> u8 {
        let index: usize = rn.into();
        self.content[index]
    }
    /// Check the stackpointer.
    pub fn is_stackpointer_valid(&self, ss: Stacksize) -> bool {
        let sp = self.content[5];
        if sp >= 0xF0 {
            return false;
        }
        match ss {
            Stacksize::_16 => sp >= 0xA0 && sp <= 0xAF,
            Stacksize::_32 => sp >= 0xA0 && sp <= 0xBF,
            Stacksize::_48 => sp >= 0xA0 && sp <= 0xCF,
            Stacksize::_64 => sp >= 0xA0 && sp <= 0xDF,
            Stacksize::NotSet => true,
        }
    }
}

impl From<RegisterNumber> for usize {
    fn from(rn: RegisterNumber) -> Self {
        match rn {
            RegisterNumber::R0 => 0,
            RegisterNumber::R1 => 1,
            RegisterNumber::R2 => 2,
            RegisterNumber::R3 => 3,
            RegisterNumber::R4 => 4,
            RegisterNumber::R5 => 5,
            RegisterNumber::R6 => 6,
            RegisterNumber::R7 => 7,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::machine::{Instruction, MP28BitWord, Register, Signal};

    #[test]
    fn test_register_block_basics() {
        let reg = Register::new();
        assert_eq!(reg.content, [0; 8]);
    }
    #[test]
    fn test_register_block_writing() {
        use crate::machine::Instruction as I;
        use crate::machine::MP28BitWord as W;

        let mut reg = Register::new();
        // All inputs empty => IN A => R0
        let inst = Instruction::empty();
        let word = MP28BitWord::empty();
        let signal = Signal::new(&word, &inst);
        reg.write(&signal, 0xAB);
        assert_eq!(reg.content[0], 0xAB);
        // MRGWS | MRGAB2 | MRGAB1 => IN B => R5
        let word = W::MRGWS | W::MRGAB2 | W::MRGAB1;
        let signal = Signal::new(&word, &inst);
        reg.write(&signal, 0xAC);
        assert_eq!(reg.content[6], 0xAC);
        // MRGWS | MRGAB3 | OP10 => IN B => R1
        let word = W::MRGWS | W::MRGAB3;
        let inst = I::OP10;
        let signal = Signal::new(&word, &inst);
        reg.write(&signal, 0xFF);
        assert_eq!(reg.content[1], 0xFF);
        // MRGAA2 | MRGAA1 | MRGAA0 => IN A => R7
        let word = W::MRGAA2 | W::MRGAA1 | W::MRGAA0;
        let inst = I::empty();
        let signal = Signal::new(&word, &inst);
        reg.write(&signal, 0xCD);
        assert_eq!(reg.content[7], 0xCD);
        // MRGAA3 | OP01 | OP00 => IN A => R3
        let word = W::MRGAA3;
        let inst = I::OP01 | I::OP00;
        let signal = Signal::new(&word, &inst);
        reg.write(&signal, 0x03);
        assert_eq!(reg.content[3], 0x03);
    }
    #[test]
    fn test_register_block_flags() {
        let mut reg = Register::new();
        let inst = Instruction::empty();
        let word = MP28BitWord::empty();
        let mut signal = Signal::new(&word, &inst);
        // All flags off by default
        assert_eq!(reg.cf(), false);
        assert_eq!(reg.zf(), false);
        assert_eq!(reg.nf(), false);
        // Update flags #1
        signal.set_co(false);
        signal.set_zo(true);
        signal.set_no(false);
        reg.write_flags(&signal);
        assert_eq!(reg.cf(), false);
        assert_eq!(reg.zf(), true);
        assert_eq!(reg.nf(), false);
        assert_eq!(reg.ief(), false);
        // Update flags #2
        reg.set_ief();
        assert_eq!(reg.cf(), false);
        assert_eq!(reg.zf(), true);
        assert_eq!(reg.nf(), false);
        assert_eq!(reg.ief(), true);
        // Update flags #3
        signal.set_co(true);
        signal.set_zo(true);
        signal.set_no(false);
        reg.write_flags(&signal);
        assert_eq!(reg.cf(), true);
        assert_eq!(reg.zf(), true);
        assert_eq!(reg.nf(), false);
        assert_eq!(reg.ief(), true);
    }
    #[test]
    fn test_register_block_output_a() {
        use crate::machine::MP28BitWord as W;

        let reg = Register {
            content: [0xF0, 0xF1, 0xF2, 0xF3, 0x12, 0x32, 0x56, 0x00],
        };
        // MRGAA3 => R0
        let inst = Instruction::empty();
        let word = W::MRGAA3;
        let signal = Signal::new(&word, &inst);
        assert_eq!(reg.doa(&signal), 0xF0);
        // MRGAA2 | MRGAA0 => R5
        let inst = Instruction::empty();
        let word = W::MRGAA2 | W::MRGAA0;
        let signal = Signal::new(&word, &inst);
        assert_eq!(reg.doa(&signal), 0x32);
    }
    #[test]
    fn test_register_block_output_b() {
        use crate::machine::MP28BitWord as W;

        let reg = Register {
            content: [0xF0, 0xF1, 0xF2, 0xF3, 0x12, 0x32, 0x56, 0x00],
        };
        // MRGAA3 (ignored) => R0
        let inst = Instruction::empty();
        let word = W::MRGAA3;
        let signal = Signal::new(&word, &inst);
        assert_eq!(reg.dob(&signal), 0xF0);
        // MRGAB1 | MRGAB0 => R3
        let inst = Instruction::empty();
        let word = W::MRGAB1 | W::MRGAB0;
        let signal = Signal::new(&word, &inst);
        assert_eq!(reg.dob(&signal), 0xF3);
    }
}
