use log::error;
use log::trace;

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
}

impl Bus {
    /// Create a new Bus.
    /// The ram is empty.
    pub fn new() -> Self {
        let ram = [0; 0xF0];
        let input_reg = [0; 4];
        let output_reg = [0; 2];
        Bus {
            ram,
            input_reg,
            output_reg,
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
        if addr <= 0xEF {
            self.ram[addr] = byte;
            trace!("RAM update: {:?}", self.ram.to_vec());
        } else if addr <= 0xFD {
            // TODO: Implement
            error!("Cannot yet write to non ram bus content. address: {}", addr);
        } else {
            self.output_reg[addr - 0xFE] = byte;
            trace!("Output register update: {:?}", self.output_reg);
        }
    }
    /// Read from the bus.
    pub fn read(&self, addr: u8) -> u8 {
        let addr = addr as usize;
        if addr <= 0xEF {
            self.ram[addr]
        } else if addr <= 0xFB {
            // TODO: Implement
            error!("Cannot yet read from non ram bus content");
            0
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
