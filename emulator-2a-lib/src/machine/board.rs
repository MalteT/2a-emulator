//! The MR2DA2 extension board and all its components.
use bitflags::bitflags;
use enum_primitive::{
    enum_from_primitive, enum_from_primitive_impl, enum_from_primitive_impl_ty, FromPrimitive,
};
use log::{trace, warn};

use std::{f32::consts::FRAC_PI_2, u8};

const MAX_FAN_RPM: usize = 4200;

/// The external board of the Minirechner 2a (MR2DA2).
///
/// ```text
///                          ┌─────────────┨ P-AI1
///     Bus                  │ ┌─────╮       0.20V
///      ┇                   └─┼+ CP1├──┐
///      ┃                   ┌─┼-    │  ○ D-CP1
///      ┃   ┌───┐    ┌────╮ │ └─────╯  │
///    F0┣━━━┽RG1┝━┳━━┽DAC1├─┴──╴○╶─────│──┨ P-AO1
///      ┃   └───┘ ┃  └────╯     D-AO1  │    2.40V
/// F1[3]┃<────────┃────────────────────┘
///      ┃         ┗━━━━━━━━━━━━━━━━━━━━━━━┫ P-DO1
///      ┃                                   214
///      ┃
///    F0┃<━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫ P-DI1
///      ┃                                   99
///      ┃   ┌────┐
///      ┃   │TEMP├──╴○╶─────┬─────────────┨ P-AI2
///      ┃   └────┘   D-AI2  │ ┌─────╮       0.00V
///      ┃                   └─┼+ CP2├──┐
///      ┃                   ┌─┼-    │  ○ D-CP2
///      ┃   ┌───┐    ┌────╮ │ └─────╯  │
///    F1┣━━━┽RG2┝━┳━━┽DAC2├─┴──╴○╶─────│──┨ P-AO2
///      ┃   └───┘ ┃  └────╯     D-AO2  │    1.33V
/// F1[4]┃<────────┃────────────────────┘
///      ┃         ┗━━━━━━━━━━━━━━━━━━━━━━━┫ P-DO2
///      ┇                                   42
/// ```
#[derive(Debug, Clone)]
pub struct Board {
    /// The 8-bit input port.
    digital_input1: u8,
    /// The 8-bit output port 1.
    digital_output1: u8,
    /// The 8-bit output port 2.
    digital_output2: u8,
    /// Temperature value as a voltage.
    // TODO: The handling of the temperature sensor is a shabby workaround, just
    // implement something useful with good defaults already!
    temp: f32,
    /// Digital Analog Status Register.
    dasr: DASR,
    /// Digital Analog Interrupt Status Register.
    daisr: DAISR,
    /// Digital Analog Interrupt Control Register.
    daicr: DAICR,
    /// Analog input ports: I1 and I2.
    analog_inputs: [f32; 2],
    /// Analog output ports: O1 and O2.
    analog_outputs: [f32; 2],
    /// Fan rpms. This is an oversimplification. The maximum fan rpm equals 4200.
    /// But this fan spins even at 0.1V supply voltage.
    /// freq(volt) = 70Hz / 2.55V * volt
    // TODO: This could be improved upon aswell. Doing this coherent to the temp sensor
    // would be nice
    fan_rpm: usize,
    /// UIO directions:
    ///
    /// - `true` => Output
    /// - `false` => Input
    uio_dir: [bool; 3],
}

bitflags! {
    /// Digital Analog Status Register
    pub struct DASR: u8 {
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
    pub struct DAISR: u8 {
        const INTERRUPT_PENDING   = 0b00001000;
        const INTERRUPT_REQUESTED = 0b00000100;
        const INTERRUPT_FF        = 0b00000010;
        const SOURCE              = 0b00000001;
    }
}

bitflags! {
    /// Digital Analog Interrupt Control Register
    pub struct DAICR: u8 {
        const IE            = 0b00100000;
        const EDGE          = 0b00010000;
        const FALLING       = 0b00001000;
        const INT_SOURCE2   = 0b00000100;
        const INT_SOURCE1   = 0b00000010;
        const INT_SOURCE0   = 0b00000001;
    }
}

impl DAICR {
    pub fn interrupt_source(&self) -> InterruptSource {
        let source = (self.contains(DAICR::INT_SOURCE2) as u8) << 2
            | (self.contains(DAICR::INT_SOURCE1) as u8) << 1
            | (self.contains(DAICR::INT_SOURCE0) as u8);
        InterruptSource::from_u8(source).expect("infallible")
    }

    pub fn set_interrupt_source(&mut self, source: InterruptSource) {
        let source = source as u8;
        let int_source_2 = source & 0b100 != 0;
        let int_source_1 = source & 0b010 != 0;
        let int_source_0 = source & 0b001 != 0;
        self.set(DAICR::INT_SOURCE2, int_source_2);
        self.set(DAICR::INT_SOURCE1, int_source_1);
        self.set(DAICR::INT_SOURCE0, int_source_0);
    }
}

enum_from_primitive! {
    /// Selected interrupt source of the MR2DA2.
    ///
    /// This is part of the DAICR, thus the source can only be
    /// changed by changing the DAICR.
    ///
    /// See [`DAICR::interrupt_source`] and [`DAICR::set_interrupt_source`] aswell.
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub enum InterruptSource {
        Disabled    = 0b000,
        Uio1        = 0b001,
        Uio2        = 0b010,
        Uio3        = 0b011,
        Comp1       = 0b100,
        Comp2       = 0b101,
        Jumper1     = 0b110,
        TachoSensor = 0b111,
    }
}

impl Board {
    /// Initialize a new Board.
    pub const fn new() -> Self {
        Board {
            dasr: DASR::empty(),
            daisr: DAISR::empty(),
            daicr: DAICR::empty(),
            digital_input1: 0,
            temp: 0.0,
            analog_inputs: [0.0; 2],
            analog_outputs: [0.0; 2],
            digital_output1: 0,
            digital_output2: 0,
            fan_rpm: 0,
            uio_dir: [false; 3],
        }
    }

    /// Set the 8-bit input port.
    pub fn set_digital_input1(&mut self, digital_input1: u8) {
        self.digital_input1 = digital_input1
    }

    /// Set the temperature value.
    pub fn set_temp(&mut self, value: f32) {
        trace!("Setting temperature to {}", value);
        if (0.0..=5.0).contains(&value) {
            self.temp = value;
        } else if value >= 0.0 {
            warn!("Temperature value > 5.0. Set to 5.0!");
            self.temp = 5.0;
        } else {
            warn!("Temperature value < 0.0. Set to 0.0!");
            self.temp = 0.0;
        }
        self.update_comp2();
    }

    /// Set the jumper J1.
    ///
    /// - `true` => Plugged in.
    /// - `false` => Unplugged.
    pub fn set_jumper1(&mut self, plugged: bool) {
        if self.daicr().interrupt_source() == InterruptSource::Jumper1 {
            if self.dasr.contains(DASR::J1) && !plugged {
                if self.daicr.contains(DAICR::FALLING) {
                    self.daisr.insert(DAISR::SOURCE);
                    self.set_int_ff();
                }
            } else if !self.dasr.contains(DASR::J1)
                && plugged
                && !self.daicr.contains(DAICR::FALLING)
            {
                self.daisr.insert(DAISR::SOURCE);
                self.set_int_ff();
            }
        }
        self.dasr.set(DASR::J1, plugged);
    }

    /// Set the jumper J2.
    ///
    /// - `true` => Plugged in.
    /// - `false` => Unplugged.
    pub fn set_jumper2(&mut self, plugged: bool) {
        self.dasr.set(DASR::J2, plugged);
    }

    /// Set analog input port I1.
    pub fn set_analog_input1(&mut self, value: f32) {
        if (0.0..=5.0).contains(&value) {
            self.analog_inputs[0] = value;
        } else if value >= 0.0 {
            warn!("I1 > 5V. Setting 5V");
            self.analog_inputs[0] = 5.0;
        } else {
            warn!("I1 < 0V. Setting 0V");
            self.analog_inputs[0] = 0.0;
        }
        self.update_comp1();
    }

    /// Set analog input port I2.
    pub fn set_analog_input2(&mut self, value: f32) {
        if (0.0..=5.0).contains(&value) {
            self.analog_inputs[1] = value;
        } else if value >= 0.0 {
            warn!("I2 > 5V. Setting 5V");
            self.analog_inputs[1] = 5.0;
        } else {
            warn!("I2 < 0V. Setting 0V");
            self.analog_inputs[1] = 0.0;
        }
        self.update_comp2();
    }

    /// Set universal input/output port UIO1.
    pub fn set_universal_input_output1(&mut self, value: bool) {
        if self.uio_dir[0] {
            return;
        }
        if self.daicr().interrupt_source() == InterruptSource::Uio1 {
            if self.dasr.contains(DASR::UIO_1) && !value {
                if self.daicr.contains(DAICR::FALLING) {
                    self.daisr.insert(DAISR::SOURCE);
                    self.set_int_ff();
                }
            } else if !self.dasr.contains(DASR::UIO_1)
                && value
                && !self.daicr.contains(DAICR::FALLING)
            {
                self.daisr.insert(DAISR::SOURCE);
                self.set_int_ff();
            }
        }
        self.dasr.set(DASR::UIO_1, value);
    }

    /// Set universal input/output port UIO2.
    pub fn set_universal_input_output2(&mut self, value: bool) {
        if self.uio_dir[1] {
            return;
        }
        if self.daicr().interrupt_source() == InterruptSource::Uio2 {
            if self.dasr.contains(DASR::UIO_2) && !value {
                if self.daicr.contains(DAICR::FALLING) {
                    self.daisr.insert(DAISR::SOURCE);
                    self.set_int_ff();
                }
            } else if !self.dasr.contains(DASR::UIO_2)
                && value
                && !self.daicr.contains(DAICR::FALLING)
            {
                self.daisr.insert(DAISR::SOURCE);
                self.set_int_ff();
            }
        }
        self.dasr.set(DASR::UIO_2, value);
    }

    /// Set universal input/output port UIO3.
    pub fn set_universal_input_output3(&mut self, value: bool) {
        if self.uio_dir[2] {
            return;
        }
        if self.daicr().interrupt_source() == InterruptSource::Uio3 {
            if self.dasr.contains(DASR::UIO_3) && !value {
                if self.daicr.contains(DAICR::FALLING) {
                    self.daisr.insert(DAISR::SOURCE);
                    self.set_int_ff();
                }
            } else if !self.dasr.contains(DASR::UIO_3)
                && value
                && !self.daicr.contains(DAICR::FALLING)
            {
                self.daisr.insert(DAISR::SOURCE);
                self.set_int_ff();
            }
        }
        self.dasr.set(DASR::UIO_3, value);
    }

    /// Set the 8-bit output port ORG1.
    pub fn set_digital_output1(&mut self, value: u8) {
        self.digital_output1 = value;
        let analog = value as f32 / 100.0;
        self.analog_outputs[0] = analog;
        self.update_comp1();
        self.fan_rpm = (MAX_FAN_RPM as f32 * analog / 2.55) as usize;
        self.dasr.insert(DASR::FAN);
    }

    /// Set the 8-bit output port ORG2.
    pub fn set_digital_output2(&mut self, value: u8) {
        self.digital_output2 = value;
        let analog = value as f32 / 100.0;
        self.analog_outputs[1] = analog;
        self.update_comp2();
    }

    /// Update the UIO Output Register.
    pub fn set_uor(&mut self, byte: u8) {
        let (uio1, uio2, uio3) = (
            byte & 0b0000_0001 == 1,
            byte & 0b0000_0010 == 2,
            byte & 0b0000_0100 == 4,
        );
        self.dasr.set(DASR::UIO_1, uio1);
        self.dasr.set(DASR::UIO_2, uio2);
        self.dasr.set(DASR::UIO_3, uio3);
    }

    /// Update the UIO Direction Register.
    pub fn set_udr(&mut self, byte: u8) {
        self.uio_dir = [
            byte & 0b0000_0001 == 1,
            byte & 0b0000_0010 == 2,
            byte & 0b0000_0100 == 4,
        ];
    }

    /// Update the Interrupt Control Register.
    pub fn set_icr(&mut self, byte: u8) {
        let rem = DAISR::INTERRUPT_PENDING | DAISR::INTERRUPT_REQUESTED | DAISR::INTERRUPT_FF;
        self.daisr.remove(rem);
        self.daicr = DAICR::from_bits_truncate(byte);
    }

    /// Delete the interrupt ff.
    pub fn delete_int_ff(&mut self) {
        self.daisr.remove(DAISR::INTERRUPT_FF);
    }

    /// Get the fan period.
    ///
    /// period(volt) = 255 - 255 / 2.55V * volt
    ///
    /// The fan period is mapped to the range \[0..255\].
    pub fn get_fan_period(&self) -> u8 {
        u8::MAX - (u8::MAX as f32 / self.fan_rpm as f32 * MAX_FAN_RPM as f32) as u8
    }

    /// Is there an interrupt?
    pub fn fetch_interrupt(&mut self) -> bool {
        if !self.daicr.contains(DAICR::IE) {
            return false;
        }
        if self.daicr().interrupt_source() == InterruptSource::TachoSensor && self.fan_rpm > 0 {
            self.set_int_ff();
            self.daisr.insert(DAISR::SOURCE);
        }
        if self.daicr.contains(DAICR::EDGE) {
            let int = self.daisr.contains(DAISR::SOURCE);
            self.daisr.remove(DAISR::SOURCE);
            int
        } else {
            self.daisr.contains(DAISR::INTERRUPT_FF)
        }
    }

    /// Set the interrupt flipflop.
    fn set_int_ff(&mut self) {
        self.daisr.insert(DAISR::INTERRUPT_FF);
    }

    /// Update comparator COMP1.
    fn update_comp1(&mut self) {
        let analog = self.digital_output1 as f32 / 100.0;
        let new_value = self.analog_inputs[0] > analog;
        if self.daicr().interrupt_source() == InterruptSource::Comp1 {
            if self.dasr.contains(DASR::COMP_DAC1) && !new_value {
                if self.daicr.contains(DAICR::FALLING) {
                    self.daisr.insert(DAISR::SOURCE);
                    self.set_int_ff();
                }
            } else if !self.dasr.contains(DASR::COMP_DAC1)
                && new_value
                && !self.daicr.contains(DAICR::FALLING)
            {
                self.daisr.insert(DAISR::SOURCE);
                self.set_int_ff();
            }
        }
        trace!("Updating comparator CP1 to {}", new_value);
        self.dasr.set(DASR::COMP_DAC1, new_value);
    }

    /// Update comparator COMP2.
    fn update_comp2(&mut self) {
        let analog = self.digital_output2 as f32 / 100.0;
        // TODO: Verify (J9)
        let comp_in = self.temp.max(self.analog_inputs[1]);
        let new_value = comp_in > analog;
        if self.daicr().interrupt_source() == InterruptSource::Comp2 {
            if self.dasr.contains(DASR::COMP_DAC2) && !new_value {
                if self.daicr.contains(DAICR::FALLING) {
                    self.daisr.insert(DAISR::SOURCE);
                    self.set_int_ff();
                }
            } else if !self.dasr.contains(DASR::COMP_DAC2)
                && new_value
                && !self.daicr.contains(DAICR::FALLING)
            {
                self.daisr.insert(DAISR::SOURCE);
                self.set_int_ff();
            }
        }
        trace!("Updating comparator CP2 to {}", new_value);
        self.dasr.set(DASR::COMP_DAC2, new_value);
        trace!("New DASR: {:0>8b}", self.dasr.bits());
    }

    /// Reset the board.
    pub fn master_reset(&mut self) {
        self.digital_output1 = 0;
        self.digital_output2 = 0;
        self.analog_outputs = [0.0; 2];
        self.temp = 0.0;
        self.daicr = DAICR::empty();
        self.fan_rpm = 0;
        self.uio_dir = [false; 3];
        // self.dasr = DASR::J2;
        // self.daisr = DAISR::empty();
    }

    /// Reset the board.
    #[deprecated = "Renamed to [`Board::master_reset`]"]
    pub fn reset(&mut self) {
        self.digital_output1 = 0;
        self.digital_output2 = 0;
        self.temp = FRAC_PI_2;
        self.dasr = DASR::J2;
        self.daisr = DAISR::empty();
        self.daicr = DAICR::empty();
        self.analog_outputs = [0.0; 2];
        self.fan_rpm = 0;
        self.uio_dir = [false; 3];
    }

    pub const fn digital_input1(&self) -> &u8 {
        &self.digital_input1
    }

    pub const fn digital_output1(&self) -> &u8 {
        &self.digital_output1
    }

    pub const fn digital_output2(&self) -> &u8 {
        &self.digital_output2
    }

    pub const fn temp(&self) -> &f32 {
        &self.temp
    }

    pub const fn dasr(&self) -> &DASR {
        &self.dasr
    }

    pub const fn daisr(&self) -> &DAISR {
        &self.daisr
    }

    pub const fn daicr(&self) -> &DAICR {
        &self.daicr
    }

    pub const fn analog_inputs(&self) -> &[f32; 2] {
        &self.analog_inputs
    }

    pub const fn analog_outputs(&self) -> &[f32; 2] {
        &self.analog_outputs
    }

    pub const fn fan_rpm(&self) -> &usize {
        &self.fan_rpm
    }

    pub const fn uio_dir(&self) -> &[bool; 3] {
        &self.uio_dir
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_dac_1() {
        let mut board = Board::new();
        let err = f32::EPSILON;
        assert_eq!(board.digital_output1, 0x00);
        board.set_digital_output1(0);
        assert_eq!(board.digital_output1, 0x00);
        board.set_digital_output1(1);
        assert_eq!(board.digital_output1, 0x01);
        assert!((board.analog_outputs[0] - 0.01).abs() < err);
        board.set_digital_output1(2);
        assert_eq!(board.digital_output1, 0x02);
        assert!((board.analog_outputs[0] - 0.02).abs() < err);
        board.set_digital_output1(99);
        assert_eq!(board.digital_output1, 0x63);
        assert!((board.analog_outputs[0] - 0.99).abs() < err);
        board.set_digital_output1(100);
        assert_eq!(board.digital_output1, 0x64);
        assert!((board.analog_outputs[0] - 1.00).abs() < err);
        board.set_digital_output1(101);
        assert_eq!(board.digital_output1, 0x65);
        assert!((board.analog_outputs[0] - 1.01).abs() < err);
        board.set_digital_output1(254);
        assert_eq!(board.digital_output1, 0xFE);
        assert!((board.analog_outputs[0] - 2.54).abs() < err);
        board.set_digital_output1(255);
        assert_eq!(board.digital_output1, 0xFF);
        assert!((board.analog_outputs[0] - 2.55).abs() < err);
    }

    #[test]
    fn test_dac_2() {
        let mut board = Board::new();
        let err = f32::EPSILON;
        assert_eq!(board.digital_output2, 0x00);
        board.set_digital_output2(0);
        assert_eq!(board.digital_output2, 0x00);
        board.set_digital_output2(1);
        assert_eq!(board.digital_output2, 0x01);
        assert!((board.analog_outputs[1] - 0.01).abs() < err);
        board.set_digital_output2(2);
        assert_eq!(board.digital_output2, 0x02);
        assert!((board.analog_outputs[1] - 0.02).abs() < err);
        board.set_digital_output2(99);
        assert_eq!(board.digital_output2, 0x63);
        assert!((board.analog_outputs[1] - 0.99).abs() < err);
        board.set_digital_output2(100);
        assert_eq!(board.digital_output2, 0x64);
        assert!((board.analog_outputs[1] - 1.00).abs() < err);
        board.set_digital_output2(101);
        assert_eq!(board.digital_output2, 0x65);
        assert!((board.analog_outputs[1] - 1.01).abs() < err);
        board.set_digital_output2(254);
        assert_eq!(board.digital_output2, 0xFE);
        assert!((board.analog_outputs[1] - 2.54).abs() < err);
        board.set_digital_output2(255);
        assert_eq!(board.digital_output2, 0xFF);
        assert!((board.analog_outputs[1] - 2.55).abs() < err);
    }

    #[test]
    fn test_comp_1() {
        let mut board = Board::new();
        assert_eq!(board.dasr.bits(), 0b0000_0000);
        board.set_digital_output1(0);
        board.set_analog_input1(0.01);
        assert_eq!(board.dasr.bits(), 0b0010_1000);
        board.set_digital_output2(0);
        board.set_analog_input2(0.01);
        assert_eq!(board.dasr.bits(), 0b0011_1000);
        board.set_digital_output1(1);
        board.set_analog_input1(0.01);
        assert_eq!(board.dasr.bits(), 0b0011_0000);
        board.set_digital_output2(1);
        board.set_analog_input2(0.01);
        assert_eq!(board.dasr.bits(), 0b0010_0000);
    }
}
