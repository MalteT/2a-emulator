/// Everything related to the bus.
use bitflags::bitflags;
use log::{trace, warn};
#[cfg(test)]
use proptest::prelude::*;
#[cfg(test)]
use proptest_derive::Arbitrary;

use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use super::{Board, Interrupt};

/// The bus used in the Minirechner 2a.
///
/// # Address usage
///
/// | address   | logic                             | `rw` |
/// |-----------|-----------------------------------|------|
/// | `00 - EF` | ram                               | `rw` |
/// | `F0 - F3` | external board                    | `rw` |
/// | `F4 - F7` | external board (hidden)           | `rw` |
/// | `F8`      | ???                               | `??` |
/// | `F9`      | Master Interrupt Control Register | `_w` |
/// | `F9`      | Master Interrupt Status Register  | `r_` |
/// | `FA - FB` | UART                              | `rw` |
/// | `FC - FF` | Input register                    | `r_` |
/// | `FC - FD` | Interrupt timer                   | `_w` |
/// | `FE - FF` | Output register                   | `_w` |
///
#[derive(Clone)]
pub struct Bus {
    ram: Ram,
    input_reg: [u8; 4],
    output_reg: [u8; 2],
    micr: MICR,
    misr: MISR,
    ucr: UCR,
    usr: USR,
    uart_send: u8,
    uart_recv: u8,
    int_timer: InterruptTimer,
    board: Board,
}

/// The ram component of the [`Bus`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ram([u8; 0xF0]);

/// The interrupt timer.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct InterruptTimer {
    enabled: bool,
    div1: usize,
    div2: usize,
    div3: usize,
}

bitflags! {
    /// Master Interrupt Control Register
    #[cfg_attr(test, derive(Arbitrary))]
    struct MICR: u8 {
        const BUS_EDGE_INTERRUPT_ENABLE   = 0b00100000;
        const BUS_LEVEL_INTERRUPT_ENABLE  = 0b00010000;
        const UART_EDGE_INTERRUPT_ENABLE  = 0b00001000;
        const UART_LEVEL_INTERRUPT_ENABLE = 0b00000100;
        const TIMER_EDGE_INTERRUPT_ENABLE = 0b00000010;
        const KEY_EDGE_INTERRUPT_ENABLE   = 0b00000001;
    }
}

bitflags! {
    /// Master Interrupt Status Register
    #[cfg_attr(test, derive(Arbitrary))]
    pub struct MISR: u8 {
        const BUS_INTERRUPT_PENDING          = 0b10000000;
        const UART_INTERUPT_PENDING          = 0b01000000;
        const TIMER_INTERRUPT_PENDING        = 0b00100000;
        const KEY_INTERRUPT_PENDING          = 0b00010000;
        const BUS_INTERRUPT_REQUEST_ACTIVE   = 0b00001000;
        const UART_INTERRUPT_REQUEST_ACTIVE  = 0b00000100;
        const TIMER_INTERRUPT_REQUEST_ACTIVE = 0b00000010;
        const KEY_INTERRUPT_REQUEST_ACTIVE   = 0b00000001;
    }
}

bitflags! {
    /// UART Control Register
    /// *This ignores the baudrate.*
    #[cfg_attr(test, derive(Arbitrary))]
    struct UCR: u8 {
        const INT_ON_RX_READY = 0b10000000;
        const INT_ON_RX_FULL  = 0b01000000;
        const INT_ON_TX_EMPTY = 0b00100000;
        const INT_ON_TX_READY = 0b00010000;
        const IGNORE_CTS      = 0b00001000;
    }
}

bitflags! {
    /// UART Status Register
    #[cfg_attr(test, derive(Arbitrary))]
    struct USR: u8 {
        const TX_READY = 0b10000000;
        const TX_EMPTY = 0b01000000;
        const NOT_CTS  = 0b00100000;
        const TX_D     = 0b00010000;
        const RX_D     = 0b00001000;
        const NOT_RTS  = 0b00000100;
        const RX_FULL  = 0b00000010;
        const RX_READY = 0b00000001;
    }
}

impl Bus {
    /// Create a new Bus.
    /// The ram is empty.
    pub const fn new() -> Self {
        let ram = Ram::new();
        let input_reg = [0; 4];
        let output_reg = [0; 2];
        let micr = MICR::empty();
        let misr = MISR::empty();
        let ucr = UCR::empty();
        let usr = USR::empty();
        let uart_send = 0;
        let uart_recv = 0;
        let int_timer = InterruptTimer::new();
        let board = Board::new();
        Bus {
            ram,
            input_reg,
            output_reg,
            micr,
            misr,
            ucr,
            usr,
            uart_send,
            uart_recv,
            int_timer,
            board,
        }
    }
    /// Reset the bus.
    ///
    /// # Note
    ///
    /// Resets:
    /// - The output register.
    /// - The external board.
    /// - MICR.
    /// - MISR.
    /// - *Not* the input register nor the ram.
    #[deprecated = "use [`Machine::cpu_reset`] or [`Machine::master_reset`]"]
    pub fn reset(&mut self) {
        self.output_reg = [0; 2];
        self.board.master_reset();
        self.micr = MICR::empty();
        self.misr = MISR::empty();
    }

    /// Reset the program execution on the bus.
    ///
    /// This resets:
    ///  - The output registers
    ///  - The MICR
    ///  - The UCR
    pub fn cpu_reset(&mut self) {
        self.output_reg = [0; 2];
        self.micr = MICR::empty();
        self.ucr = UCR::empty();
    }

    /// Reset the bus.
    ///
    /// On top of the [`Bus::cpu_reset`], the following will be reset:
    ///  - The input register
    ///  - The interrupt timer config
    pub fn master_reset(&mut self) {
        self.cpu_reset();
        self.input_reg = [0; 4];
        self.int_timer.reset();
    }

    /// Fill the ram with zeros.
    ///
    /// # Example
    /// ```
    /// # use emulator_2a_lib::machine::Bus;
    /// # let mut bus = Bus::new();
    ///
    /// bus.write(0x11, 42);
    /// # assert_eq!(bus.read(0x11), 42);
    /// bus.reset_ram();
    /// assert_eq!(bus.read(0x11), 0);
    /// ```
    pub fn reset_ram(&mut self) {
        self.ram.reset();
    }

    /// Get mutable access to the Master Interrupt Status Register.
    pub(crate) fn misr_mut(&mut self) -> &mut MISR {
        &mut self.misr
    }

    /// Write to the bus
    pub fn write(&mut self, addr: u8, byte: u8) {
        let addr = addr as usize;
        trace!("Update 0x{:>02X} = 0x{:>02X}", addr, byte);
        if addr <= 0xEF {
            self.ram[addr] = byte;
        } else if addr == 0xF0 {
            self.board.set_digital_output1(byte);
        } else if addr == 0xF1 {
            self.board.set_digital_output2(byte);
        } else if addr == 0xF2 {
            match (byte & 0b1100_0000) >> 6 {
                0b00 => self.board.set_uor(byte),
                0b01 => warn!("Writing 0b11****** to 0xF2 does nothing"),
                0b10 => self.board.set_udr(byte),
                0b11 => self.board.set_icr(byte),
                _ => unreachable!(),
            }
        } else if addr == 0xF3 {
            self.board.delete_int_ff();
        } else if addr == 0xF4 {
            warn!("Writing to 0xF4 does nothing! This feature might be implemented in the future, but as of now, the MR2DA2 board is very restricted.");
        } else if addr == 0xF5 {
            warn!("Writing to 0xF5 does nothing! This feature might be implemented in the future, but as of now, the MR2DA2 board is very restricted.");
        } else if addr == 0xF6 {
            warn!("Writing to 0xF6 does nothing! This feature might be implemented in the future, but as of now, the MR2DA2 board is very restricted.");
        } else if addr == 0xF7 {
            warn!("Writing to 0xF7 does nothing! This feature might be implemented in the future, but as of now, the MR2DA2 board is very restricted.");
        } else if addr == 0xF8 {
            // 0xF8 serves no purpose
            warn!("Writing to 0xF8 does nothing!. Ask Werner Dreher...");
        } else if addr == 0xF9 {
            self.micr = MICR::from_bits_truncate(byte);
        } else if addr == 0xFA {
            self.uart_send = byte;
        } else if addr == 0xFB {
            self.ucr = UCR::from_bits_truncate(byte);
        } else if addr == 0xFC {
            let lower = byte as usize;
            let orig = self.int_timer.div3;
            self.int_timer.div3 = (orig & 0xFF00) + lower;
        } else if addr == 0xFD {
            let top_bit_set = (byte & 0b1000_0000) == 0b1000_0000;
            if top_bit_set {
                self.int_timer.enabled = byte & 0b0001_0000 == 0b0001_0000;
                let div2_select = (byte & 0b0000_1100) >> 2;
                self.int_timer.div2 = match div2_select {
                    0b00 => 1,
                    0b01 => 10,
                    0b10 => 100,
                    0b11 => 1000,
                    _ => unreachable!(),
                };
                let div1_select = byte & 0b0000_0011;
                self.int_timer.div2 = match div1_select {
                    0b00 => 1,
                    0b01 => 16,
                    0b10 => 256,
                    0b11 => 4096,
                    _ => unreachable!(),
                };
            } else {
                let upper = (byte as usize & 0b0111_1111) << 7;
                let orig = self.int_timer.div3;
                self.int_timer.div3 = upper + (orig & 0b0111_1111);
            }
        } else if addr == 0xFE {
            self.output_reg[0] = byte;
        } else if addr == 0xFF {
            self.output_reg[1] = byte;
        }
    }
    /// Read from the bus.
    pub fn read(&self, addr: u8) -> u8 {
        let addr = addr as usize;
        if addr <= 0xEF {
            self.ram[addr]
        } else if addr == 0xF0 {
            *self.board.digital_input1()
        } else if addr == 0xF1 {
            self.board.dasr().bits()
        } else if addr == 0xF2 {
            self.board.get_fan_period()
        } else if addr == 0xF3 {
            self.board.daisr().bits()
        } else if addr == 0xF4 {
            warn!("Reading from 0xF4 does nothing! This feature might be implemented in the future, but as of now, the MR2DA2 board is very restricted.");
            0
        } else if addr == 0xF5 {
            warn!("Reading from 0xF5 does nothing! This feature might be implemented in the future, but as of now, the MR2DA2 board is very restricted.");
            0
        } else if addr == 0xF6 {
            warn!("Reading from 0xF6 does nothing! This feature might be implemented in the future, but as of now, the MR2DA2 board is very restricted.");
            0
        } else if addr == 0xF7 {
            warn!("Reading from 0xF7 does nothing! This feature might be implemented in the future, but as of now, the MR2DA2 board is very restricted.");
            0
        } else if addr == 0xF8 {
            // 0xF8 serves no purpose
            warn!("Reading from 0xF8 does nothing!. Ask Werner Dreher...");
            0
        } else if addr == 0xF9 {
            self.misr.bits()
        } else if addr == 0xFA {
            self.uart_recv
        } else if addr == 0xFB {
            self.usr.bits()
        } else {
            self.input_reg[addr - 0xFC]
        }
    }
    /// Set input register `FC`.
    pub fn input_fc(&mut self, byte: u8) {
        self.input_reg[0] = byte;
    }
    /// Set input register `FD`.
    pub fn input_fd(&mut self, byte: u8) {
        self.input_reg[1] = byte;
    }
    /// Set input register `FE`.
    pub fn input_fe(&mut self, byte: u8) {
        self.input_reg[2] = byte;
    }
    /// Set input register `FF`.
    pub fn input_ff(&mut self, byte: u8) {
        self.input_reg[3] = byte;
    }
    /// Get output register `FE`.
    pub fn output_fe(&self) -> u8 {
        self.output_reg[0]
    }
    /// Get output register `FF`.
    pub fn output_ff(&self) -> u8 {
        self.output_reg[1]
    }
    /// Is anything on the bus triggering a level interrupt?
    ///
    /// TODO: Implement
    pub fn get_level_interrupt(&mut self) -> Option<Interrupt> {
        warn!("Bus Interrupts are not implemented yet");
        None
        //if self.micr.contains(MICR::UART_LEVEL_INTERRUPT_ENABLE) {
        //    None
        //} else if self.micr.contains(MICR::BUS_LEVEL_INTERRUPT_ENABLE) {
        //    None
        //} else {
        //    None
        //}
    }
    /// Did anything on the bus trigger an edge interrupt?
    ///
    /// # Note:
    /// Level intterupts can also be triggered by the timer and by key!
    /// These are not checked here.
    /// TODO: Implement
    pub fn take_edge_interrupt(&mut self) -> Option<Interrupt> {
        warn!("Bus Interrupts are not implemented yet");
        None
        //if self.micr.contains(MICR::UART_EDGE_INTERRUPT_ENABLE) {
        //    None
        //} else if self.micr.contains(MICR::BUS_EDGE_INTERRUPT_ENABLE) {
        //    None
        //} else {
        //    None
        //}
    }
    /// Get read access to the board.
    pub fn board(&self) -> &Board {
        &self.board
    }

    /// Get mutable access to the connected [`Board`].
    pub fn board_mut(&mut self) -> &mut Board {
        &mut self.board
    }

    /// Is key edge interrupt enabled?
    pub fn is_key_edge_int_enabled(&self) -> bool {
        self.micr.contains(MICR::KEY_EDGE_INTERRUPT_ENABLE)
    }
    /// Is timer edge interrupt enabled?
    #[allow(dead_code)]
    pub fn is_timer_edge_int_enabled(&self) -> bool {
        self.micr.contains(MICR::TIMER_EDGE_INTERRUPT_ENABLE)
    }
    /// Get the contents of the main memory.
    ///
    /// The main memory ranges from 0x00 - 0xEF.
    pub fn memory(&self) -> &[u8; 0xF0] {
        &self.ram
    }

    /// Get mutable access to the memory connected to the bus.
    ///
    /// # Example
    /// ```
    /// # use emulator_2a_lib::machine::Bus;
    /// let mut bus = Bus::new();
    ///
    /// let memory = bus.memory_mut();
    /// memory[0] = 123;
    /// memory[42] = 76;
    /// memory[0xEF] = 0xFF;
    ///
    /// assert_eq!(bus.read(0), 123);
    /// assert_eq!(bus.read(42), 76);
    /// assert_eq!(bus.read(0xEF), 0xFF);
    /// ```
    pub fn memory_mut(&mut self) -> &mut [u8; 0xF0] {
        &mut self.ram
    }

    /// Did anything trigger an interrupt in the UART?
    #[allow(dead_code)]
    fn has_uart_interrupt(&self) -> bool {
        if self.ucr.contains(UCR::INT_ON_RX_READY) {
            self.usr.contains(USR::RX_READY)
        } else if self.ucr.contains(UCR::INT_ON_RX_FULL) {
            self.usr.contains(USR::RX_FULL)
        } else if self.ucr.contains(UCR::INT_ON_TX_EMPTY) {
            self.usr.contains(USR::TX_EMPTY)
        } else if self.ucr.contains(UCR::INT_ON_TX_READY) {
            self.usr.contains(USR::TX_READY)
        } else {
            false
        }
    }
    /// Did anything trigger an interrupt on the MR2DA2?
    ///
    /// # TODO
    ///
    /// This is not implemented (yet).
    #[allow(dead_code)]
    fn fetch_mr2da2_interrupt(&mut self) -> bool {
        self.board.fetch_interrupt()
    }
}

impl Ram {
    /// Initialize a new set of Ram.
    ///
    /// # Note:
    /// Do not download RAM!
    pub const fn new() -> Self {
        Ram([0; 0xF0])
    }
    /// Reset the ram to all zeros.
    pub fn reset(&mut self) {
        self.0 = [0; 0xF0]
    }
}

impl InterruptTimer {
    /// Create a new, disabled interrupt timer.
    pub const fn new() -> Self {
        InterruptTimer {
            enabled: false,
            div1: 0,
            div2: 0,
            div3: 0,
        }
    }
    /// Reset the configuration of the interrupt timer.
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

impl fmt::Debug for Bus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Bus")
            .field("ram", &self.ram.to_vec())
            .finish()
    }
}

impl Deref for Ram {
    type Target = [u8; 0xF0];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Ram {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
prop_compose! {
    fn arbitrary_ram()(num in any::<u8>()) -> Ram {
        Ram([num; 0xF0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Bus {
        prop_compose! {
            pub fn arbitrary()(
                ram in arbitrary_ram(),
                input_reg in any::<[u8; 4]>(),
                output_reg in any::<[u8; 2]>(),
                micr in any::<MICR>(),
                misr in any::<MISR>(),
                ucr in any::<UCR>(),
                usr in any::<USR>(),
                uart_send in any::<u8>(),
                uart_recv in any::<u8>(),
                int_timer in any::<InterruptTimer>(),
                board in Board::arbitrary(),
            ) -> Bus {
                Bus { ram, input_reg, output_reg, micr, misr, ucr, usr, uart_send, uart_recv, int_timer, board }
            }
        }
    }

    proptest! {
        #[test]
        fn ram_is_reset_correctly(mut bus in Bus::arbitrary()) {
            bus.reset_ram();
            assert_eq!(bus.ram, Bus::new().ram);
        }

        #[test]
        fn ram_is_untouched_by_cpu_and_master_reset(mut bus in Bus::arbitrary()) {
            let pristine = bus.clone();
            bus.cpu_reset();
            assert_eq!(bus.ram, pristine.ram);
            bus.master_reset();
            assert_eq!(bus.ram, pristine.ram);
        }

        #[test]
        fn input_registers_are_reset_by_master_reset_only(mut bus in Bus::arbitrary()) {
            let pristine = bus.clone();
            bus.cpu_reset();
            assert_eq!(bus.input_reg, pristine.input_reg);
            bus.master_reset();
            assert_eq!(bus.input_reg, Bus::new().input_reg);
        }

        #[test]
        fn output_registers_are_reset_by_cpu_reset(mut bus in Bus::arbitrary()) {
            let pristine = bus.clone();
            bus.cpu_reset();
            assert_eq!(bus.output_reg, Bus::new().output_reg);
            // Let's make sure the master reset does that aswell
            bus = pristine;
            bus.master_reset();
            assert_eq!(bus.output_reg, Bus::new().output_reg);
        }

        #[test]
        fn micr_is_reset_by_cpu_reset(mut bus in Bus::arbitrary()) {
            bus.cpu_reset();
            assert_eq!(bus.micr, Bus::new().micr);
        }

        #[test]
        fn ucr_is_reset_by_cpu_reset(mut bus in Bus::arbitrary()) {
            bus.cpu_reset();
            assert_eq!(bus.ucr, Bus::new().ucr);
        }

        #[test]
        fn interrupt_timer_settings_are_reset_by_master_reset_only(mut bus in Bus::arbitrary()) {
            let pristine = bus.clone();
            bus.cpu_reset();
            assert_eq!(bus.int_timer, pristine.int_timer);
            bus.master_reset();
            assert_eq!(bus.int_timer, Bus::new().int_timer);
        }
    }

    #[test]
    fn test_bus_ram() {
        let mut bus = Bus::new();
        // Test write
        bus.write(0x00, 0x11);
        bus.write(0xE1, 0x12);
        bus.write(0xEF, 0x13);
        assert_eq!(bus.ram[0x00], 0x11);
        assert_eq!(bus.ram[0xE1], 0x12);
        assert_eq!(bus.ram[0xEF], 0x13);
        // Test read
        bus.write(0x11, 0x14);
        assert_eq!(bus.read(0x00), 0x11);
        assert_eq!(bus.read(0x01), 0x00);
        assert_eq!(bus.read(0xE1), 0x12);
        assert_eq!(bus.read(0xEF), 0x13);
        assert_eq!(bus.read(0x11), 0x14);
    }

    #[test]
    fn test_bus_input_reg() {
        let mut bus = Bus::new();
        bus.input_fc(123);
        bus.input_fd(124);
        bus.input_fe(125);
        bus.input_ff(126);
        // Verify inputing
        assert_eq!(123, bus.input_reg[0]);
        assert_eq!(124, bus.input_reg[1]);
        assert_eq!(125, bus.input_reg[2]);
        assert_eq!(126, bus.input_reg[3]);
        // Verify reading
        assert_eq!(bus.read(0xFC), 123);
        assert_eq!(bus.read(0xFD), 124);
        assert_eq!(bus.read(0xFE), 125);
        assert_eq!(bus.read(0xFF), 126);
    }

    #[test]
    fn test_bus_output_reg() {
        let mut bus = Bus::new();
        bus.write(0xFE, 12);
        bus.write(0xFF, 0xFF);
        // Verify writing
        assert_eq!(bus.output_reg[0], 12);
        assert_eq!(bus.output_reg[1], 0xFF);
        // Verify reading
        assert_eq!(bus.output_fe(), 12);
        assert_eq!(bus.output_ff(), 0xFF);
    }
}
