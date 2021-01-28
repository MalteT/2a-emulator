//! Everything needed to operate the microprogram ram.
use bitflags::bitflags;
#[cfg(test)]
use proptest_derive::Arbitrary;

use std::ops::Index;

/// The microprogram ram.
///
/// Containing all microprogram words used by the
/// Minirechner 2a as defined in the documentation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct MicroprogramRam {
    /// Current index into the [`Self::CONTENT`].
    current_index: usize,
}

bitflags! {
    /// A Word stored in the microprogram ram
    ///
    /// This defines signal that are used throughout the machine.
    pub struct Word: u32 {
        const MAC3       = 0b00001000000000000000000000000000;
        const MAC2       = 0b00000100000000000000000000000000;
        const MAC1       = 0b00000010000000000000000000000000;
        const MAC0       = 0b00000001000000000000000000000000;
        const NA4        = 0b00000000100000000000000000000000;
        const NA3        = 0b00000000010000000000000000000000;
        const NA2        = 0b00000000001000000000000000000000;
        const NA1        = 0b00000000000100000000000000000000;
        const NA0        = 0b00000000000010000000000000000000;
        const BUSWR      = 0b00000000000001000000000000000000;
        const BUSEN      = 0b00000000000000100000000000000000;
        const MRGAA3     = 0b00000000000000010000000000000000;
        const MRGAA2     = 0b00000000000000001000000000000000;
        const MRGAA1     = 0b00000000000000000100000000000000;
        const MRGAA0     = 0b00000000000000000010000000000000;
        const MRGAB3     = 0b00000000000000000001000000000000;
        const MRGAB2     = 0b00000000000000000000100000000000;
        const MRGAB1     = 0b00000000000000000000010000000000;
        const MRGAB0     = 0b00000000000000000000001000000000;
        const MRGWS      = 0b00000000000000000000000100000000;
        const MRGWE      = 0b00000000000000000000000010000000;
        const MALUIA     = 0b00000000000000000000000001000000;
        const MALUIB     = 0b00000000000000000000000000100000;
        const MALUS3     = 0b00000000000000000000000000010000;
        const MALUS2     = 0b00000000000000000000000000001000;
        const MALUS1     = 0b00000000000000000000000000000100;
        const MALUS0     = 0b00000000000000000000000000000010;
        const MCHFLG     = 0b00000000000000000000000000000001;
        const ERROR_STOP = 0b00000000000000000000000000000000;
    }
}

impl MicroprogramRam {
    /// Contents of the MicroprogramRam.
    pub const CONTENT: [Word; 512] = include!("microprogram_ram_content.rs");
    /// Create a new MicroprogramRam with initial address of zero.
    ///
    /// # Example
    ///
    /// ```
    /// # use emulator_2a_lib::machine::{MicroprogramRam};
    /// let mut ram = MicroprogramRam::new();
    ///
    /// assert_eq!(ram.get_address(), 0);
    /// ```
    pub const fn new() -> Self {
        MicroprogramRam { current_index: 0 }
    }
    /// Get the currently active word.
    ///
    /// # Example
    ///
    /// ```
    /// # use emulator_2a_lib::machine::{Word, MicroprogramRam};
    /// let mut ram = MicroprogramRam::new();
    /// let word = MicroprogramRam::CONTENT[0];
    ///
    /// assert_eq!(*ram.get_word(), word);
    /// ```
    pub const fn get_word(&self) -> &Word {
        &Self::CONTENT[self.current_index]
    }
    /// Get the current address that is selected in the microprogram ram.
    ///
    /// # Example
    ///
    /// ```
    /// # use emulator_2a_lib::machine::MicroprogramRam;
    /// let ram = MicroprogramRam::new();
    /// assert_eq!(ram.get_address(), 0);
    /// ```
    pub const fn get_address(&self) -> usize {
        self.current_index
    }
    /// Select the next word according to the given address.
    ///
    /// # Example
    ///
    /// ```
    /// # use emulator_2a_lib::machine::MicroprogramRam;
    /// let mut ram = MicroprogramRam::new();
    /// assert_eq!(ram.get_address(), 0);
    ///
    /// ram.set_address(120);
    /// assert_eq!(ram.get_address(), 120);
    /// ```
    pub fn set_address(&mut self, address: usize) {
        self.current_index = address;
    }
    /// Reset the current address of the microprogram ram.
    ///
    /// # Example
    ///
    /// ```
    /// # use emulator_2a_lib::machine::MicroprogramRam;
    /// let mut ram = MicroprogramRam::new();
    /// ram.set_address(42);
    ///
    /// ram.reset();
    /// assert_eq!(ram.get_address(), 0);
    /// ```
    pub fn reset(&mut self) {
        self.current_index = 0;
    }
}

impl Index<usize> for MicroprogramRam {
    type Output = Word;
    fn index(&self, index: usize) -> &Word {
        &Self::CONTENT[index]
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn microprogram_resets_correctly(mut ram: MicroprogramRam) {
            ram.reset();
            assert_eq!(ram, MicroprogramRam::new());
        }
    }
}
