/// Everything needed for a higher level abstraction over the raw [`Machine`].
use crate::Machine;

/// A higher level abstraction over the [`Machine`].
///
/// Using this is recommended over using the raw [`Machine`].
///
/// TODO: Examples
pub struct MachineInterface {
    machine: Machine,
}

impl MachineInterface {
    pub const fn new() -> Self {
        MachineInterface {
            machine: Machine::new(),
        }
    }

    /// Get a reference to the underlying machine.
    ///
    /// # Example
    ///
    /// ```
    /// # use emulator_2a_lib::{MachineInterface};
    /// let mut interface = MachineInterface::new();
    /// interface.set_input_ff(42);
    ///
    /// let input_reg_ff = interface.machine().bus().read(0xFF);
    /// assert_eq!(input_reg_ff, 42);
    /// ```
    pub const fn machine(&self) -> &Machine {
        &self.machine
    }

    /// Get mutable access to the underlying machine.
    ///
    /// **Note**: Use this as a last resort only. You should always prefer
    /// the existing methods for mutating the machine.
    ///
    /// TODO: Examples
    pub fn machine_mut(&mut self) -> &mut Machine {
        &mut self.machine
    }

    /// Emulate a rising CLK edge.
    ///
    /// TODO: Examples
    pub fn next_cycle(&mut self) {
        self.machine.trigger_clock_edge()
    }

    /// Set the content of the input register FC to `number`.
    ///
    /// TODO: Examples
    pub fn set_input_fc(&mut self, number: u8) {
        self.machine_mut().bus_mut().input_fc(number)
    }

    /// Set the content of the input register FD to `number`.
    ///
    /// TODO: Examples
    pub fn set_input_fd(&mut self, number: u8) {
        self.machine_mut().bus_mut().input_fd(number)
    }

    /// Set the content of the input register FE to `number`.
    ///
    /// TODO: Examples
    pub fn set_input_fe(&mut self, number: u8) {
        self.machine_mut().bus_mut().input_fe(number)
    }

    /// Set the content of the input register FF to `number`.
    ///
    /// TODO: Examples
    pub fn set_input_ff(&mut self, number: u8) {
        self.machine_mut().bus_mut().input_ff(number)
    }

    /// Trigger an interrupt by key.
    ///
    /// TODO: Examples
    pub fn trigger_key_interrupt(&mut self) {
        self.machine_mut().trigger_key_edge_interrupt()
    }

    /// Load the given bytes of a program into the main memory.
    ///
    /// The memory will be filled starting at address zero. All bytes
    /// will be written consecutively.
    ///
    /// # Example
    ///
    /// ```
    /// # use emulator_2a_lib::MachineInterface;
    /// let mut interface = MachineInterface::new();
    /// let mut program = vec![23, 24, 25, 26, 0, 28];
    ///
    /// interface.fill_memory(program.drain(..));
    ///
    /// assert_eq!(interface.machine().bus().read(0), 23);
    /// assert_eq!(interface.machine().bus().read(5), 28);
    ///
    /// interface.fill_memory((0_u8..0xF0).by_ref());
    ///
    /// assert_eq!(interface.machine().bus().memory()[0], 0);
    /// assert_eq!(interface.machine().bus().memory()[0xEF], 0xEF);
    /// ```
    ///
    /// # Panic
    ///
    /// This method will panic if more than `0xF0` (`240`) bytes are supplied.
    pub fn fill_memory<I>(&mut self, bytes: I)
    where
        I: IntoIterator<Item = u8>,
    {
        bytes.into_iter().enumerate().for_each(|(address, byte)| {
            self.machine_mut().bus_mut().memory_mut()[address] = byte;
        });
    }
}
