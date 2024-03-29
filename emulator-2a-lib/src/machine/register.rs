use bitflags::bitflags;
use enum_primitive::{enum_from_primitive, enum_from_primitive_impl, enum_from_primitive_impl_ty};
#[cfg(test)]
use proptest_derive::Arbitrary;

use std::ops::{Index, IndexMut};

use crate::parser;

/// The register block.
/// Containing `R0` through `R7`
///
/// # Example
///
/// ```
/// # use emulator_2a_lib::machine::{Register, RegisterNumber};
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
///
/// # Note
///
/// Even though the register R4 (flag register) is only using bits 0 to 3, bits 4 to 7 are not
/// guaranteed to be zero and will be kept by all flag operations.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct Register {
    content: [u8; 8],
}

enum_from_primitive! {
    /// All possible register.
    ///
    /// This is only useful to index [`Register`].
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[cfg_attr(test, derive(Arbitrary))]
    pub enum RegisterNumber {
        R0 = 0,
        R1,
        R2,
        R3,
        R4,
        R5,
        R6,
        R7,
    }
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
    /// # use emulator_2a_lib::machine::{Register, RegisterNumber};
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
    /// # use emulator_2a_lib::machine::{Register, RegisterNumber};
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
    /// # use emulator_2a_lib::machine::{Register, RegisterNumber, Flags};
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
    /// # use emulator_2a_lib::machine::{Register};
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
    /// # use emulator_2a_lib::machine::{Register};
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
    /// # use emulator_2a_lib::machine::{Register};
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
    /// # use emulator_2a_lib::machine::{Register};
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
    /// # use emulator_2a_lib::machine::{Register, Flags};
    /// let mut reg = Register::new();
    /// reg.set_interrupt_enable_flag(true);
    ///
    /// assert!(reg.flags().contains(Flags::INTERRUPT_ENABLE_FLAG));
    /// ```
    pub fn set_interrupt_enable_flag(&mut self, val: bool) {
        if val {
            self.content[4] |= Flags::INTERRUPT_ENABLE_FLAG.bits()
        } else {
            self.content[4] &= !Flags::INTERRUPT_ENABLE_FLAG.bits()
        }
    }
    /// Set the carry flag to `val`.
    ///
    /// # Example
    ///
    /// ```
    /// # use emulator_2a_lib::machine::{Register, Flags};
    /// let mut reg = Register::new();
    /// reg.set_carry_flag(true);
    ///
    /// assert!(reg.flags().contains(Flags::CARRY_FLAG));
    /// ```
    pub fn set_carry_flag(&mut self, val: bool) {
        if val {
            self.content[4] |= Flags::CARRY_FLAG.bits()
        } else {
            self.content[4] &= !Flags::CARRY_FLAG.bits()
        }
    }
    /// Set the zero flag to `val`.
    ///
    /// # Example
    ///
    /// ```
    /// # use emulator_2a_lib::machine::{Register, Flags};
    /// let mut reg = Register::new();
    /// reg.set_zero_flag(true);
    ///
    /// assert!(reg.flags().contains(Flags::ZERO_FLAG));
    /// ```
    pub fn set_zero_flag(&mut self, val: bool) {
        if val {
            self.content[4] |= Flags::ZERO_FLAG.bits()
        } else {
            self.content[4] &= !Flags::ZERO_FLAG.bits()
        }
    }
    /// Set the negative flag to `val`.
    ///
    /// # Example
    ///
    /// ```
    /// # use emulator_2a_lib::machine::{Register, Flags};
    /// let mut reg = Register::new();
    /// reg.set_negative_flag(true);
    ///
    /// assert!(reg.flags().contains(Flags::NEGATIVE_FLAG));
    /// ```
    pub fn set_negative_flag(&mut self, val: bool) {
        if val {
            self.content[4] |= Flags::NEGATIVE_FLAG.bits()
        } else {
            self.content[4] &= !Flags::NEGATIVE_FLAG.bits()
        }
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
    /// # use emulator_2a_lib::machine::{Register, Flags};
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
        self.content[4] &= !Flags::all().bits();
        self.content[4] |= new_flags.bits()
    }
    /// Get register content for the given [`RegisterNumber`].
    ///
    /// # Example
    ///
    /// ```
    /// # use emulator_2a_lib::machine::{Register, RegisterNumber};
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
    /// # use emulator_2a_lib::machine::{Register, RegisterNumber};
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
    /// # use emulator_2a_lib::machine::{Register, RegisterNumber, Flags};
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
    /// # use emulator_2a_lib::machine::{Register, RegisterNumber};
    /// let mut reg = Register::new();
    /// reg.set(RegisterNumber::R1, 42);
    /// reg.reset();
    ///
    /// assert_eq!(*reg.get(RegisterNumber::R1), 0);
    /// ```
    pub fn reset(&mut self) {
        self.content = [0; 8];
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

impl From<parser::Register> for RegisterNumber {
    fn from(reg: parser::Register) -> Self {
        match reg {
            parser::Register::R0 => RegisterNumber::R0,
            parser::Register::R1 => RegisterNumber::R1,
            parser::Register::R2 => RegisterNumber::R2,
            parser::Register::R3 => RegisterNumber::R3,
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
    use proptest::prelude::*;

    use crate::machine::{Register, RegisterNumber};

    proptest! {
        #[test]
        fn write_to_register_should_persist(number: RegisterNumber, val in 0..255_u8) {
            let mut register = Register::new();
            register.set(number, val);
            assert_eq!(*register.get(number), val);
        }

        #[test]
        fn registers_are_reset_correctly(mut registers: Register) {
            registers.reset();
            assert_eq!(registers, Register::new());
        }
    }

    #[test]
    fn test_register_block_basics() {
        let reg = Register::new();
        assert_eq!(reg.content, [0; 8]);
    }
}
