use bitflags::bitflags;
use log::error;
use log::trace;
use log::warn;

use std::fmt;

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
// # TODO: Interrupt Timer
// # TODO: Master Interrupt Control / Status Register
// # TODO: UART
// # TODO: External board
#[derive(Clone)]
pub struct Bus {
    ram: [u8; 0xF0],
    input_reg: [u8; 4],
    output_reg: [u8; 2],
    micr: MICR,
    misr: MISR,
    ucr: UCR,
    usr: USR,
    dac1: u8,
    dac2: u8,
    uart_send: u8,
    uart_recv: u8,
    dasr: DASR,
    daisr: DAISR,
    daicr: DAICR,
}

bitflags! {
    /// Master Interrupt Control Register
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
    struct MISR: u8 {
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

bitflags! {
    /// Digital Analog Status Register
    struct DASR: u8 {
        const J2        = 0b10000000;
        const J1        = 0b01000000;
        const FAN       = 0b00100000;
        const COMP_DAC2 = 0b00010000;
        const COMP_DAC1 = 0b00001000;
        const UIO_3     = 0b00000100;
        const UIO_2     = 0b00000010;
        const UIO_1     = 0b00000001;
    }
}

bitflags! {
    /// Digital Analog Interrupt Status Register
    struct DAISR: u8 {
        const INTERRUPT_PENDING   = 0b00001000;
        const INTERRUPT_REQUESTED = 0b00000100;
        const INTERRUPT_FF        = 0b00000010;
        const SOURCE              = 0b00000001;
    }
}

bitflags! {
    /// Digital Analog Interrupt Control Register
    struct DAICR: u8 {
        const IE      = 0b00100000;
        const EDGE    = 0b00010000;
        const FALLING = 0b00001000;
    }
}

impl Bus {
    /// Create a new Bus.
    /// The ram is empty.
    pub fn new() -> Self {
        let ram = [0; 0xF0];
        let input_reg = [0; 4];
        let output_reg = [0; 2];
        let micr = MICR::empty();
        let misr = MISR::empty();
        let ucr = UCR::empty();
        let usr = USR::empty();
        let dac1 = 0;
        let dac2 = 0;
        let uart_send = 0;
        let uart_recv = 0;
        let dasr = DASR::empty();
        let daisr = DAISR::empty();
        let daicr = DAICR::empty();
        Bus {
            ram,
            input_reg,
            output_reg,
            micr,
            misr,
            ucr,
            usr,
            dac1,
            dac2,
            uart_send,
            uart_recv,
            dasr,
            daisr,
            daicr,
        }
    }
    /// Reset the output registers.
    ///
    /// *Not* the input register nor the ram.
    pub fn reset(&mut self) {
        self.output_reg = [0; 2];
    }
    /// Write to the bus
    pub fn write(&mut self, addr: u8, byte: u8) {
        let addr = addr as usize;
        trace!("Update 0x{:>02X} = 0x{:>02X}", addr, byte);
        if addr <= 0xEF {
            self.ram[addr] = byte;
        } else if addr == 0xF0 {
            self.dac1 = byte;
        } else if addr == 0xF1 {
            self.dac2 = byte;
        } else if addr == 0xF2 {
            error!("Cannot yet write to 0xF2")
        } else if addr == 0xF3 {
            error!("Cannot yet write to 0xF3")
        } else if addr == 0xF4 {
            error!("Cannot yet write to 0xF4")
        } else if addr == 0xF5 {
            error!("Cannot yet write to 0xF5")
        } else if addr == 0xF6 {
            error!("Cannot yet write to 0xF6")
        } else if addr == 0xF7 {
            error!("Cannot yet write to 0xF7")
        } else if addr == 0xF8 {
            // 0xF8 serves no purpose
        } else if addr == 0xF9 {
            self.micr = MICR::from_bits_truncate(byte);
        } else if addr == 0xFA {
            self.uart_send = byte;
        } else if addr == 0xFB {
            self.ucr = UCR::from_bits_truncate(byte);
        } else if addr == 0xFC {
            // TODO
            error!("Cannot yet write to 0xFC")
        } else if addr == 0xFD {
            // TODO
            error!("Cannot yet write to 0xFD")
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
            warn!("0xF0 is unsupported!");
            0
        } else if addr == 0xF1 {
            self.dasr.bits()
        } else if addr == 0xF2 {
            error!("Cannot read from 0xF2 yet.");
            0
        } else if addr == 0xF3 {
            self.daisr.bits()
        } else if addr == 0xF4 {
            error!("Cannot read from 0xF4 yet.");
            0
        } else if addr == 0xF5 {
            error!("Cannot read from 0xF5 yet.");
            0
        } else if addr == 0xF6 {
            error!("Cannot read from 0xF6 yet.");
            0
        } else if addr == 0xF7 {
            error!("Cannot read from 0xF7 yet.");
            0
        } else if addr == 0xF8 {
            error!("Cannot read from 0xF8 yet.");
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
    /// Did anything on the bus trigger a level interrupt?
    pub fn has_level_int(&self) -> bool {
        if self.micr.contains(MICR::UART_LEVEL_INTERRUPT_ENABLE) {
            self.has_uart_interrupt()
        } else if self.micr.contains(MICR::BUS_LEVEL_INTERRUPT_ENABLE) {
            self.has_mr2da2_interrupt()
        } else {
            false
        }
    }
    /// Did anything on the bus trigger an edge interrupt?
    pub fn has_edge_int(&self) -> bool {
        if self.micr.contains(MICR::UART_EDGE_INTERRUPT_ENABLE) {
            self.has_uart_interrupt()
        } else if self.micr.contains(MICR::BUS_EDGE_INTERRUPT_ENABLE) {
            self.has_mr2da2_interrupt()
        } else {
            false
        }
    }
    /// Is key edge interrupt enabled?
    pub fn is_key_edge_int_enabled(&self) -> bool {
        self.micr.contains(MICR::KEY_EDGE_INTERRUPT_ENABLE)
    }
    /// Is timer edge interrupt enabled?
    pub fn is_timer_edge_int_enabled(&self) -> bool {
        self.micr.contains(MICR::TIMER_EDGE_INTERRUPT_ENABLE)
    }
    /// Did anything trigger an interrupt in the UART?
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
    fn has_mr2da2_interrupt(&self) -> bool {
        if self.daicr.contains(DAICR::EDGE) {
            self.daisr.contains(DAISR::INTERRUPT_FF)
        } else {
            if self.daicr.contains(DAICR::FALLING) {
                unimplemented!()
            } else {
                unimplemented!()
            }
        }
    }
}

impl fmt::Debug for Bus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Bus")
            .field("ram", &self.ram.to_vec())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
