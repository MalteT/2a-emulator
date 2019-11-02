use bitflags::bitflags;
use log::warn;

const MAX_FAN_RPM: usize = 4200;

/// The external board of the Minirechner 2a.
#[derive(Debug, Clone)]
pub struct Board {
    /// The 8-bit input port.
    pub(super) irg: u8,
    /// The 8-bit output port 1.
    pub(super) org1: u8,
    /// The 8-bit output port 2.
    pub(super) org2: u8,
    /// Temperature value.
    pub(super) temp: f32,
    /// Digital Analog Status Register.
    pub(super) dasr: DASR,
    /// Digital Analog Interrupt Status Register.
    pub(super) daisr: DAISR,
    /// Digital Analog Interrupt Control Register.
    pub(super) daicr: DAICR,
    /// Analog input ports: I1 and I2.
    pub(super) analog_inputs: [f32; 2],
    /// Analog output ports: O1 and O2.
    pub(super) analog_outputs: [f32; 2],
    /// Fan rpms. This is an oversimplification. The maximum fan rpm equals 4200.
    /// But this fan spins even at 0.1V supply voltage.
    pub(super) fan_rpm: usize,
    /// Interrupt source (DA-ICR[0-2].
    int_source: u8,
    /// UIO directions:
    ///
    /// - `true` => Output
    /// - `false` => Input
    pub(super) uio_dir: [bool; 3],
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
    const IE      = 0b00100000;
    const EDGE    = 0b00010000;
    const FALLING = 0b00001000;
}
}

impl Board {
    /// Initialize a new Board.
    pub fn new() -> Self {
        let dasr = DASR::empty();
        let daisr = DAISR::empty();
        let daicr = DAICR::empty();
        let irg = 0;
        let org1 = 0;
        let org2 = 0;
        let temp = 0.0;
        let analog_inputs = [0.0; 2];
        let analog_outputs = [0.0; 2];
        let fan_rpm = 0;
        let int_source = 0;
        let uio_dir = [false; 3];
        Board {
            dasr,
            daisr,
            daicr,
            irg,
            temp,
            analog_inputs,
            analog_outputs,
            org1,
            org2,
            fan_rpm,
            int_source,
            uio_dir,
        }
    }
    /// Set the 8-bit input port.
    pub fn set_irg(&mut self, value: u8) {
        self.irg = value;
    }
    /// Set the temperature value.
    pub fn set_temp(&mut self, value: f32) {
        if value >= 0.0 && value <= 5.0 {
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
    pub fn set_j1(&mut self, plugged: bool) {
        if self.int_source == 0b110 {
            if self.dasr.contains(DASR::J1) && !plugged {
                if self.daicr.contains(DAICR::FALLING) {
                    self.daisr.insert(DAISR::SOURCE);
                    self.set_int_ff();
                }
            } else if !self.dasr.contains(DASR::J1) && plugged {
                if !self.daicr.contains(DAICR::FALLING) {
                    self.daisr.insert(DAISR::SOURCE);
                    self.set_int_ff();
                }
            }
        }
        self.dasr.set(DASR::J1, plugged);
    }
    /// Set the jumper J2.
    ///
    /// - `true` => Plugged in.
    /// - `false` => Unplugged.
    pub fn set_j2(&mut self, plugged: bool) {
        self.dasr.set(DASR::J2, plugged);
    }
    /// Set analog input port I1.
    pub fn set_i1(&mut self, value: f32) {
        if value >= 0.0 && value <= 5.0 {
            self.analog_inputs[0] = value;
        } else if value >= 0.0 {
            warn!("I1 > 5V. Setting 5V");
            self.analog_inputs[0] = 5.0;
        } else {
            warn!("I1 < 0V. Setting 0V");
            self.analog_inputs[0] = 0.0;
        }
    }
    /// Set analog input port I2.
    pub fn set_i2(&mut self, value: f32) {
        if value >= 0.0 && value <= 5.0 {
            self.analog_inputs[1] = value;
        } else if value >= 0.0 {
            warn!("I2 > 5V. Setting 5V");
            self.analog_inputs[1] = 5.0;
        } else {
            warn!("I2 < 0V. Setting 0V");
            self.analog_inputs[1] = 0.0;
        }
    }
    /// Set universal input/output port UIO1.
    pub fn set_uio1(&mut self, value: bool) {
        if self.uio_dir[0] {
            return;
        }
        if self.int_source == 0b001 {
            if self.dasr.contains(DASR::UIO_1) && !value {
                if self.daicr.contains(DAICR::FALLING) {
                    self.daisr.insert(DAISR::SOURCE);
                    self.set_int_ff();
                }
            } else if !self.dasr.contains(DASR::UIO_1) && value {
                if !self.daicr.contains(DAICR::FALLING) {
                    self.daisr.insert(DAISR::SOURCE);
                    self.set_int_ff();
                }
            }
        }
        self.dasr.set(DASR::UIO_1, value);
    }
    /// Set universal input/output port UIO2.
    pub fn set_uio2(&mut self, value: bool) {
        if self.uio_dir[1] {
            return;
        }
        if self.int_source == 0b010 {
            if self.dasr.contains(DASR::UIO_2) && !value {
                if self.daicr.contains(DAICR::FALLING) {
                    self.daisr.insert(DAISR::SOURCE);
                    self.set_int_ff();
                }
            } else if !self.dasr.contains(DASR::UIO_2) && value {
                if !self.daicr.contains(DAICR::FALLING) {
                    self.daisr.insert(DAISR::SOURCE);
                    self.set_int_ff();
                }
            }
        }
        self.dasr.set(DASR::UIO_2, value);
    }
    /// Set universal input/output port UIO3.
    pub fn set_uio3(&mut self, value: bool) {
        if self.uio_dir[2] {
            return;
        }
        if self.int_source == 0b011 {
            if self.dasr.contains(DASR::UIO_3) && !value {
                if self.daicr.contains(DAICR::FALLING) {
                    self.daisr.insert(DAISR::SOURCE);
                    self.set_int_ff();
                }
            } else if !self.dasr.contains(DASR::UIO_3) && value {
                if !self.daicr.contains(DAICR::FALLING) {
                    self.daisr.insert(DAISR::SOURCE);
                    self.set_int_ff();
                }
            }
        }
        self.dasr.set(DASR::UIO_3, value);
    }
    /// Set the 8-bit output port ORG1.
    pub fn set_org1(&mut self, value: u8) {
        self.org1 = value;
        let analog = value as f32 / 100.0;
        self.analog_outputs[0] = analog;
        self.update_comp1();
        self.fan_rpm = (MAX_FAN_RPM as f32 * analog / 2.55) as usize;
        self.dasr.insert(DASR::FAN);
    }
    /// Set the 8-bit output port ORG2.
    pub fn set_org2(&mut self, value: u8) {
        self.org2 = value;
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
        self.int_source = byte & 0b0000_0111;
    }
    /// Delete the interrupt ff.
    pub fn delete_int_ff(&mut self) {
        self.daisr.remove(DAISR::INTERRUPT_FF);
    }
    /// Get the fan period.
    ///
    /// The fan period is mapped to the range \[0..255\].
    pub fn get_fan_period(&self) -> u8 {
        let period = if self.fan_rpm <= 1 {
            60.0
        } else {
            60.0 / self.fan_rpm as f32
        };
        (period * 255.0 / 60.0) as u8
    }
    /// Is there an interrupt?
    pub fn fetch_interrupt(&mut self) -> bool {
        if !self.daicr.contains(DAICR::IE) {
            return false;
        }
        if self.int_source == 0b111 && self.fan_rpm > 0 {
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
        let analog = self.org1 as f32 / 100.0;
        let new_value = self.analog_inputs[0] > analog;
        if self.int_source == 0b100 {
            if self.dasr.contains(DASR::COMP_DAC1) && !new_value {
                if self.daicr.contains(DAICR::FALLING) {
                    self.daisr.insert(DAISR::SOURCE);
                    self.set_int_ff();
                }
            } else if !self.dasr.contains(DASR::COMP_DAC1) && new_value {
                if !self.daicr.contains(DAICR::FALLING) {
                    self.daisr.insert(DAISR::SOURCE);
                    self.set_int_ff();
                }
            }
        }
        self.dasr.set(DASR::COMP_DAC1, new_value);
    }
    /// Update comparator COMP2.
    fn update_comp2(&mut self) {
        let analog = self.org2 as f32 / 100.0;
        // TODO: Verify (J9)
        let comp_in = self.temp.max(self.analog_inputs[1]);
        let new_value = comp_in > analog;
        if self.int_source == 0b101 {
            if self.dasr.contains(DASR::COMP_DAC2) && !new_value {
                if self.daicr.contains(DAICR::FALLING) {
                    self.daisr.insert(DAISR::SOURCE);
                    self.set_int_ff();
                }
            } else if !self.dasr.contains(DASR::COMP_DAC2) && new_value {
                if !self.daicr.contains(DAICR::FALLING) {
                    self.daisr.insert(DAISR::SOURCE);
                    self.set_int_ff();
                }
            }
        }
        self.dasr.set(DASR::COMP_DAC2, new_value);
    }
}
