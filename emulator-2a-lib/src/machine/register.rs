use bitflags::bitflags;
use enum_primitive::{enum_from_primitive, enum_from_primitive_impl, enum_from_primitive_impl_ty};

use std::ops::{Index, IndexMut};

use super::Signals;

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
#[derive(Debug, Clone)]
pub struct Register {
    content: [u8; 8],
}

enum_from_primitive! {
    /// All possible register.
    ///
    /// This is only useful to index [`Register`].
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    /// Get current data output A of the register.
    #[deprecated]
    pub fn doa(&self, signal: &Signals) -> u8 {
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
    pub fn dob(&self, signal: &Signals) -> u8 {
        let (b2, b1, b0) = if signal.mrgab3() {
            (false, signal.op11(), signal.op10())
        } else {
            (signal.mrgab2(), signal.mrgab1(), signal.mrgab0())
        };
        let addr = ((b2 as usize) << 2) + ((b1 as usize) << 1) + (b0 as usize);
        self.content[addr]
    }
    /// Derive the selected register from the given [`Signals`]s.
    #[deprecated]
    pub fn get_selected(signal: &Signals) -> RegisterNumber {
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

impl RegisterNumber {
    #[cfg(test)]
    fn strategy() -> impl proptest::strategy::Strategy<Value = RegisterNumber> {
        use proptest::prelude::*;
        prop_oneof![
            Just(RegisterNumber::R0),
            Just(RegisterNumber::R1),
            Just(RegisterNumber::R2),
            Just(RegisterNumber::R3),
            Just(RegisterNumber::R4),
            Just(RegisterNumber::R5),
            Just(RegisterNumber::R6),
            Just(RegisterNumber::R7),
        ]
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use crate::machine::{Register, RegisterNumber};

    proptest! {
        #[test]
        fn write_to_register_should_persist(number in RegisterNumber::strategy(), val in 0..255_u8) {
            let mut register = Register::new();
            register.set(number, val);
            assert_eq!(*register.get(number), val);
        }
    }

    #[test]
    fn test_register_block_basics() {
        let reg = Register::new();
        assert_eq!(reg.content, [0; 8]);
    }
}
