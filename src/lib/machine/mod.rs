//! The actual machine and all its components.
use derive_builder::Builder;
use log::trace;

use std::ops::Deref;

mod alu;
mod board;
mod bus;
mod instruction;
mod microprogram_ram;
mod raw;
mod register;

pub use alu::{AluInput, AluOutput, AluSelect};
pub use board::{Board, InterruptSource, DAICR, DAISR, DASR};
pub use bus::Bus;
pub use instruction::{Instruction, InstructionRegister};
pub use microprogram_ram::{MicroprogramRam, Word};
pub(crate) use raw::Interrupt;
pub use raw::{RawMachine, Signals, State};
pub use register::{Flags, Register, RegisterNumber};

/// A higher level abstraction over the [`RawMachine`].
///
/// Using this is recommended over using the [`RawMachine`].
///
/// TODO: Examples
pub struct Machine {
    /// Underlying oily, rusty, ductaped [`RawMachine`].
    raw: RawMachine,
    /// Currently active [`StepMode`].
    step_mode: StepMode,
}

impl Machine {
    pub fn new(config: MachineConfig) -> Self {
        let mut m = Machine {
            raw: RawMachine::new(),
            step_mode: StepMode::Real,
        };
        m.set_input_fc(config.input_fc);
        m.set_input_fd(config.input_fd);
        m.set_input_fe(config.input_fe);
        m.set_input_ff(config.input_ff);
        m.set_digital_input1(config.digital_input1);
        m.set_temp(config.temp);
        m.set_jumper1(config.jumper1);
        m.set_jumper2(config.jumper2);
        m.set_analog_input1(config.analog_input1);
        m.set_analog_input2(config.analog_input2);
        m.set_universal_input_output1(config.universal_input_output1);
        m.set_universal_input_output2(config.universal_input_output2);
        m.set_universal_input_output3(config.universal_input_output3);
        m
    }

    /// Get a reference to the underlying raw machine.
    ///
    /// # Example
    ///
    /// ```
    /// # use emulator_2a_lib::machine::{Machine, MachineConfig};
    /// let mut machine = Machine::new(MachineConfig::default());
    /// machine.set_input_ff(42);
    ///
    /// let input_reg_ff = machine.bus().read(0xFF);
    /// assert_eq!(input_reg_ff, 42);
    /// ```
    #[deprecated(note = "Machine implements Deref now!")]
    pub const fn raw(&self) -> &RawMachine {
        &self.raw
    }

    /// Get the currently active [`StepMode`].
    pub const fn step_mode(&self) -> StepMode {
        self.step_mode
    }

    /// Get mutable access to the underlying raw machine.
    ///
    /// **Note**: Use this as a last resort only. You should always prefer
    /// the existing methods for mutating the machine.
    ///
    /// TODO: Examples
    pub fn raw_mut(&mut self) -> &mut RawMachine {
        &mut self.raw
    }

    /// Emulate a rising CLK edge.
    ///
    /// TODO: Examples
    pub fn trigger_key_clock(&mut self) {
        match self.step_mode {
            StepMode::Assembly => {
                // Start the next instruction
                while self.is_instruction_done() && self.state() == State::Running {
                    self.raw_mut().trigger_clock_edge()
                }
                // Finish this instruction
                while !self.is_instruction_done() && self.state() == State::Running {
                    self.raw_mut().trigger_clock_edge()
                }
            }
            StepMode::Real => self.raw_mut().trigger_clock_edge(),
        }
    }

    /// Set the content of the input register FC to `number`.
    ///
    /// TODO: Examples
    pub fn set_input_fc(&mut self, number: u8) {
        self.raw_mut().bus_mut().input_fc(number)
    }

    /// Set the content of the input register FD to `number`.
    ///
    /// TODO: Examples
    pub fn set_input_fd(&mut self, number: u8) {
        self.raw_mut().bus_mut().input_fd(number)
    }

    /// Set the content of the input register FE to `number`.
    ///
    /// TODO: Examples
    pub fn set_input_fe(&mut self, number: u8) {
        self.raw_mut().bus_mut().input_fe(number)
    }

    /// Set the content of the input register FF to `number`.
    ///
    /// TODO: Examples
    pub fn set_input_ff(&mut self, number: u8) {
        self.raw_mut().bus_mut().input_ff(number)
    }

    /// Trigger the continue key.
    ///
    /// This will return the State to [`Running`](State::Running) if it was [`Stopped`](State::Stopped).
    pub fn trigger_key_continue(&mut self) {
        self.raw_mut().trigger_key_continue()
    }

    /// Trigger an interrupt by key.
    ///
    /// TODO: Examples
    pub fn trigger_key_interrupt(&mut self) {
        self.raw_mut().trigger_key_edge_interrupt()
    }

    /// Set the value of the digital input P-DI1.
    ///
    /// This input port is part of the MR2DA2 extension board.
    pub fn set_digital_input1(&mut self, digital_input1: u8) {
        self.raw_mut()
            .bus_mut()
            .board_mut()
            .set_digital_input1(digital_input1)
    }

    /// Set the output voltage of the temperature sensor.
    ///
    /// The temperature sensor is part of the MR2DA2 extension board.
    /// It's output voltage is fed into the comparator CP2 and powers
    /// the led D-AI2. This is equivalent to setting the analog input
    /// voltage of port P-AI2
    pub fn set_temp(&mut self, temp: f32) {
        self.raw_mut().bus_mut().board_mut().set_temp(temp)
    }

    /// Plug jumper J1 into the extension board MR2DA2?
    ///
    /// This is a universal jumper. It's current state can be read
    /// from the DA-SR status register of the MR2DA2 extension board.
    pub fn set_jumper1(&mut self, jumper1: bool) {
        self.raw_mut().bus_mut().board_mut().set_jumper1(jumper1)
    }

    /// Plug jumper J2 into the extension board MR2DA2?
    ///
    /// This is a universal jumper. It's current state can be read
    /// from the DA-SR status register of the MR2DA2 extension board.
    pub fn set_jumper2(&mut self, jumper2: bool) {
        self.raw_mut().bus_mut().board_mut().set_jumper2(jumper2)
    }

    /// Set the voltage at the analog input port P-AI1.
    ///
    /// The P-AI1 is part of the extension board MR2DA2. The voltage
    /// will be fed into the comparator CP1.
    pub fn set_analog_input1(&mut self, analog_input1: f32) {
        self.raw_mut()
            .bus_mut()
            .board_mut()
            .set_analog_input1(analog_input1)
    }

    /// Set the voltage at the analog input port P-AI2.
    ///
    /// The P-AI2 is part of the extension board MR2DA2. The voltage
    /// will be fed into the comparator CP2 and power the the led D-AI2.
    /// It's effect is the same as setting the voltage of the
    /// temperature sensor
    pub fn set_analog_input2(&mut self, analog_input2: f32) {
        self.raw_mut()
            .bus_mut()
            .board_mut()
            .set_analog_input2(analog_input2)
    }

    /// Set the universal I/O port UIO1.
    ///
    /// The UIO1 port is located on the MR2DA2 extension board and
    /// can be used to in- or output a bit. Setting this does not
    /// configure the port as an input port. A program has to do that.
    pub fn set_universal_input_output1(&mut self, uio1: bool) {
        self.raw_mut()
            .bus_mut()
            .board_mut()
            .set_universal_input_output1(uio1)
    }

    /// Set the universal I/O port UIO2.
    ///
    /// See [`set_universal_input_output1`](Machine::set_universal_input_output1) for more.
    pub fn set_universal_input_output2(&mut self, uio2: bool) {
        self.raw_mut()
            .bus_mut()
            .board_mut()
            .set_universal_input_output2(uio2)
    }

    /// Set the universal I/O port UIO3.
    ///
    /// See [`set_universal_input_output1`](Machine::set_universal_input_output1) for more.
    pub fn set_universal_input_output3(&mut self, uio3: bool) {
        self.raw_mut()
            .bus_mut()
            .board_mut()
            .set_universal_input_output3(uio3)
    }

    pub fn set_step_mode(&mut self, step_mode: StepMode) {
        self.step_mode = step_mode
    }

    /// Load the given bytes of a program into the main memory.
    ///
    /// The memory will be filled starting at address zero. All bytes
    /// will be written consecutively.
    ///
    /// # Example
    ///
    /// ```
    /// # use emulator_2a_lib::machine::{Machine, MachineConfig};
    /// let mut machine = Machine::new(MachineConfig::default());
    /// let mut program = vec![23, 24, 25, 26, 0, 28];
    ///
    /// machine.fill_memory(program.drain(..));
    ///
    /// assert_eq!(machine.bus().read(0), 23);
    /// assert_eq!(machine.bus().read(5), 28);
    ///
    /// machine.fill_memory((0_u8..0xF0).by_ref());
    ///
    /// assert_eq!(machine.bus().memory()[0], 0);
    /// assert_eq!(machine.bus().memory()[0xEF], 0xEF);
    /// ```
    ///
    /// # Panic
    ///
    /// This method will panic if more than `0xF0` (`240`) bytes are supplied.
    #[deprecated(note = "use load_program instead")]
    pub fn fill_memory<I>(&mut self, bytes: I)
    where
        I: IntoIterator<Item = u8>,
    {
        bytes.into_iter().enumerate().for_each(|(address, byte)| {
            self.raw_mut().bus_mut().memory_mut()[address] = byte;
        });
    }

    /// TODO: Documentation
    pub fn load_program<'a, I>(&mut self, bytes: I)
    where
        I: Iterator<Item = &'a u8>,
    {
        trace!("Loading program into memory");
        bytes.enumerate().for_each(|(address, byte)| {
            self.raw_mut().bus_mut().memory_mut()[address] = *byte;
        });
        self.reset();
    }

    /// Reset the machine.
    ///
    /// TODO: Explain
    pub fn reset(&mut self) {
        self.raw_mut().reset()
    }
}

impl Deref for Machine {
    type Target = RawMachine;
    fn deref(&self) -> &RawMachine {
        &self.raw
    }
}

/// Configuration for the machine.
/// These values will be set initially before the emulation starts.
///
/// Each of these values has a corresponding `set_` method on [`Machine`].
/// See these methods for more information.
///
/// # Builder
///
/// For ease of use, the [`MachineConfigBuilder`] can be used
///
/// ```
/// # use emulator_2a_lib::machine::{MachineConfig, MachineConfigBuilder};
/// let mut config1 = MachineConfig::default();
/// config1.jumper1 = true;
///
/// let config2 = MachineConfigBuilder::default()
///     .jumper1(true)
///     .build()
///     .expect("This is always infallible");
///
/// assert_eq!(config1, config2);
/// ```
#[derive(Debug, Clone, Default, PartialEq, Builder)]
#[builder(default)]
pub struct MachineConfig {
    pub digital_input1: u8,
    pub temp: f32,
    pub jumper1: bool,
    pub jumper2: bool,
    pub analog_input1: f32,
    pub analog_input2: f32,
    pub universal_input_output1: bool,
    pub universal_input_output2: bool,
    pub universal_input_output3: bool,
    pub input_fc: u8,
    pub input_fd: u8,
    pub input_fe: u8,
    pub input_ff: u8,
}

/// Possible step modes for execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StepMode {
    /// Execute one word per rising clock edge. This is the default.
    Real,
    /// Execute one assembly instruction for every rising clock edge.
    /// The underlying machine is still executing every single word, but
    /// this gives more coarse-grained control to the user.
    Assembly,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_program_loading() {
        let mut machine = Machine::new(MachineConfig::default());
        machine.load_program([42].iter());
        assert_eq!(machine.bus().memory()[0], 42);
    }
}
