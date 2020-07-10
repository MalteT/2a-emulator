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
}
