use bitflags::bitflags;
use log::warn;
use parser2a::asm::Stacksize;

use std::ops::{Index, IndexMut};

use crate::machine::Signal;

/// The register block.
/// Containing `R0` through `R7`
///
/// # Example
///
/// ```
/// # use emulator_2a_lib::{Register, RegisterNumber};
/// let mut reg = Register::new();
/// assert_eq!(*reg.get(RegisterNumber::R3), 0);
///
/// // Set the negative flag
/// reg.set_negative_flag(true);
/// assert!(reg.negative_flag());
///
/// // Set R0 to 42.
/// reg.set(RegisterNumber::R0, 42);
/// assert_eq!(*reg.get(RegisterNumber::R0), 42);
///
/// // Reset it!
/// reg.reset();
/// assert_eq!(*reg.get(RegisterNumber::R0), 0);
/// assert_eq!(reg.negative_flag(), false);
/// ```
#[derive(Debug, Clone)]
pub struct Register {
    content: [u8; 8],
}

/// All possible register.
///
/// This is only useful to index [`Register`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

bitflags! {
    /// Flag bitmask for R4
    pub struct Flags: u8 {
        const CARRY_FLAG = 0b0000_0001;
        const ZERO_FLAG = 0b0000_0010;
        const NEGATIVE_FLAG = 0b0000_0100;
        const INTERRUPT_ENABLE_FLAG = 0b0000_1000;
    }
}

impl Register {
    /// Create a new register block.
    ///
    /// All registers are initialized with zero.
    ///
    /// # Example
    ///
    /// ```
    /// # use emulator_2a_lib::{Register, RegisterNumber};
    /// let reg = Register::new();
    ///
    /// assert_eq!(*reg.get(RegisterNumber::R2), 0);
    /// ```
    pub const fn new() -> Self {
        let content = [0; 8];
        Register { content }
    }
    /// Get the content of all registers.
    ///
    /// # Example
    ///
    /// ```
    /// # use emulator_2a_lib::{Register, RegisterNumber};
    /// let mut reg = Register::new();
    /// reg.set_zero_flag(true);
    /// reg.set(RegisterNumber::R7, 123);
    ///
    /// let content = reg.content();
    /// assert_eq!(content, &[
    ///     0, 0, 0, 0, 2, 0, 0, 123
    /// ]);
    /// ```
    pub const fn content(&self) -> &[u8; 8] {
        &self.content
    }
    /// Get the currently active flags.
    ///
    /// # Example
    ///
    /// ```
    /// # use emulator_2a_lib::{Register, RegisterNumber, Flags};
    /// let mut reg = Register::new();
    /// reg.set_interrupt_enable_flag(true);
    /// reg.set_negative_flag(true);
    ///
    /// let flags = reg.flags();
    /// assert!(flags.contains(Flags::INTERRUPT_ENABLE_FLAG));
    /// assert!(flags.contains(Flags::NEGATIVE_FLAG));
    /// assert!(!flags.contains(Flags::ZERO_FLAG));
    /// ```
    pub const fn flags(&self) -> Flags {
        Flags::from_bits_truncate(self.content[4])
    }
    /// Get the carry flag (CF).
    ///
    /// # Example
    ///
    /// ```
    /// # use emulator_2a_lib::{Register};
    /// let mut reg = Register::new();
    /// assert_eq!(reg.carry_flag(), false);
    ///
    /// reg.set_carry_flag(true);
    /// assert_eq!(reg.carry_flag(), true);
    /// ```
    pub const fn carry_flag(&self) -> bool {
        self.flags().contains(Flags::CARRY_FLAG)
    }
    /// Get the zero flag (ZF).
    ///
    /// # Example
    ///
    /// ```
    /// # use emulator_2a_lib::{Register};
    /// let mut reg = Register::new();
    /// assert_eq!(reg.zero_flag(), false);
    ///
    /// reg.set_zero_flag(true);
    /// assert_eq!(reg.zero_flag(), true);
    /// ```
    pub const fn zero_flag(&self) -> bool {
        self.flags().contains(Flags::ZERO_FLAG)
    }
    /// Get the negative flag (NF).
    ///
    /// # Example
    ///
    /// ```
    /// # use emulator_2a_lib::{Register};
    /// let mut reg = Register::new();
    /// assert_eq!(reg.negative_flag(), false);
    ///
    /// reg.set_negative_flag(true);
    /// assert_eq!(reg.negative_flag(), true);
    /// ```
    pub const fn negative_flag(&self) -> bool {
        self.flags().contains(Flags::NEGATIVE_FLAG)
    }
    /// Get the interrupt enable flag (IEF).
    ///
    /// # Example
    ///
    /// ```
    /// # use emulator_2a_lib::{Register};
    /// let mut reg = Register::new();
    /// assert_eq!(reg.interrupt_enable_flag(), false);
    ///
    /// reg.set_interrupt_enable_flag(true);
    /// assert_eq!(reg.interrupt_enable_flag(), true);
    /// ```
    pub const fn interrupt_enable_flag(&self) -> bool {
        self.flags().contains(Flags::INTERRUPT_ENABLE_FLAG)
    }
    /// Set the interrupt enable flag to `val`.
    ///
    /// # Example
    ///
    /// ```
    /// # use emulator_2a_lib::{Register, Flags};
    /// let mut reg = Register::new();
    /// reg.set_interrupt_enable_flag(true);
    ///
    /// assert!(reg.flags().contains(Flags::INTERRUPT_ENABLE_FLAG));
    /// ```
    pub fn set_interrupt_enable_flag(&mut self, val: bool) {
        let mut new_flag = self.flags();
        new_flag.set(Flags::INTERRUPT_ENABLE_FLAG, val);
        self.content[4] = new_flag.bits()
    }
    /// Set the carry flag to `val`.
    ///
    /// # Example
    ///
    /// ```
    /// # use emulator_2a_lib::{Register, Flags};
    /// let mut reg = Register::new();
    /// reg.set_carry_flag(true);
    ///
    /// assert!(reg.flags().contains(Flags::CARRY_FLAG));
    /// ```
    pub fn set_carry_flag(&mut self, val: bool) {
        let mut new_flag = self.flags();
        new_flag.set(Flags::CARRY_FLAG, val);
        self.content[4] = new_flag.bits()
    }
    /// Set the zero flag to `val`.
    ///
    /// # Example
    ///
    /// ```
    /// # use emulator_2a_lib::{Register, Flags};
    /// let mut reg = Register::new();
    /// reg.set_zero_flag(true);
    ///
    /// assert!(reg.flags().contains(Flags::ZERO_FLAG));
    /// ```
    pub fn set_zero_flag(&mut self, val: bool) {
        let mut new_flag = self.flags();
        new_flag.set(Flags::ZERO_FLAG, val);
        self.content[4] = new_flag.bits()
    }
    /// Set the negative flag to `val`.
    ///
    /// # Example
    ///
    /// ```
    /// # use emulator_2a_lib::{Register, Flags};
    /// let mut reg = Register::new();
    /// reg.set_negative_flag(true);
    ///
    /// assert!(reg.flags().contains(Flags::NEGATIVE_FLAG));
    /// ```
    pub fn set_negative_flag(&mut self, val: bool) {
        let mut new_flag = self.flags();
        new_flag.set(Flags::NEGATIVE_FLAG, val);
        self.content[4] = new_flag.bits()
    }
    /// Set the flags register (R4) to the given `new_flags`.
    ///
    /// If you just want to update a single flag, using one of the following is recommened:
    /// - [`Register::set_carry_flag`]
    /// - [`Register::set_zero_flag`]
    /// - [`Register::set_negative_flag`]
    /// - [`Register::set_interrupt_enable_flag`]
    ///
    /// # Example
    ///
    /// ```
    /// # use emulator_2a_lib::{Register, Flags};
    /// let mut reg = Register::new();
    /// reg.set_interrupt_enable_flag(true);
    ///
    /// let new_flags = Flags::NEGATIVE_FLAG | Flags::CARRY_FLAG;
    /// reg.set_flags(new_flags);
    ///
    /// assert!(reg.negative_flag());
    /// assert!(reg.carry_flag());
    /// assert_eq!(reg.interrupt_enable_flag(), false);
    /// ```
    pub fn set_flags(&mut self, new_flags: Flags) {
        self.content[4] = new_flags.bits();
    }
    /// Get register content for the given [`RegisterNumber`].
    ///
    /// # Example
    ///
    /// ```
    /// # use emulator_2a_lib::{Register, RegisterNumber};
    /// let mut reg = Register::new();
    /// reg.set_carry_flag(true);
    /// reg.set(RegisterNumber::R5, 42);
    ///
    /// let r5 = reg.get(RegisterNumber::R5);
    /// assert_eq!(*r5, 42);
    ///
    /// // Not recommended, use Register::flags() instead.
    /// let flags = reg.get(RegisterNumber::R4);
    /// assert_eq!(*flags, 0b0001);
    /// ```
    pub fn get(&self, rn: RegisterNumber) -> &u8 {
        let index: usize = rn.into();
        &self.content[index]
    }
    /// Get mutable access to the content of the register specified by the given
    /// [`RegisterNumber`].
    ///
    /// # Example
    ///
    /// ```
    /// # use emulator_2a_lib::{Register, RegisterNumber};
    /// let mut reg = Register::new();
    /// let r0 = reg.get_mut(RegisterNumber::R0);
    ///
    /// *r0 = 13;
    /// assert_eq!(*reg.get(RegisterNumber::R0), 13);
    /// ```
    pub fn get_mut(&mut self, rn: RegisterNumber) -> &mut u8 {
        let index: usize = rn.into();
        &mut self.content[index]
    }
    /// Write the given `value` to the given [`RegisterNumber`].
    ///
    /// # Example
    ///
    /// ```
    /// # use emulator_2a_lib::{Register, RegisterNumber, Flags};
    /// let mut reg = Register::new();
    /// reg.set(RegisterNumber::R6, 202);
    /// // Not recommend, use Register::set_flags() instead.
    /// reg.set(RegisterNumber::R4, 0b1111);
    ///
    /// let r6 = reg.get(RegisterNumber::R6);
    /// assert_eq!(*r6, 202);
    ///
    /// let flags = reg.flags();
    /// assert!(flags.contains(Flags::ZERO_FLAG));
    /// assert!(flags.contains(Flags::NEGATIVE_FLAG));
    /// ```
    pub fn set(&mut self, reg: RegisterNumber, value: u8) {
        let reg: usize = reg.into();
        self.content[reg] = value;
    }
    /// Clear all registers.
    ///
    /// This will set them to zero.
    ///
    /// # Example
    ///
    /// ```
    /// # use emulator_2a_lib::{Register, RegisterNumber};
    /// let mut reg = Register::new();
    /// reg.set(RegisterNumber::R1, 42);
    /// reg.reset();
    ///
    /// assert_eq!(*reg.get(RegisterNumber::R1), 0);
    /// ```
    pub fn reset(&mut self) {
        self.content = [0; 8];
    }

    /// Get current data output A of the register.
    #[deprecated]
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
    #[deprecated]
    pub fn dob(&self, signal: &Signal) -> u8 {
        let (b2, b1, b0) = if signal.mrgab3() {
            (false, signal.op11(), signal.op10())
        } else {
            (signal.mrgab2(), signal.mrgab1(), signal.mrgab0())
        };
        let addr = ((b2 as usize) << 2) + ((b1 as usize) << 1) + (b0 as usize);
        self.content[addr]
    }
    /// Derive the selected register from the given [`Signal`]s.
    #[deprecated]
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
    #[deprecated]
    pub fn write(&mut self, signal: &Signal, value: u8) {
        let selected: usize = Register::get_selected(signal).into();
        self.content[selected] = value;
    }
    /// Update flags in R4.
    #[deprecated]
    pub fn write_flags(&mut self, signal: &Signal) {
        // Persistent IEF
        let mut value = (self.interrupt_enable_flag() as u8) << 3;
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
    /// Check the stackpointer.
    #[deprecated]
    pub fn is_stackpointer_valid(&self, ss: Stacksize) -> bool {
        let sp = self.content[5];
        if sp >= 0xF0 {
            return false;
        }
        let valid = match ss {
            Stacksize::_16 => sp <= 0xD0 || sp >= 0xDF,
            Stacksize::_32 => sp <= 0xC0 || sp >= 0xCF,
            Stacksize::_48 => sp <= 0xB0 || sp >= 0xBF,
            Stacksize::_64 => sp <= 0xA0 || sp >= 0xAF,
            Stacksize::NotSet => true,
        };
        if !valid {
            warn!("Stackpointer got invalid!")
        }
        valid
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

impl Index<RegisterNumber> for Register {
    type Output = u8;
    fn index(&self, idx: RegisterNumber) -> &Self::Output {
        self.get(idx)
    }
}

impl IndexMut<RegisterNumber> for Register {
    fn index_mut(&mut self, idx: RegisterNumber) -> &mut Self::Output {
        self.get_mut(idx)
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
