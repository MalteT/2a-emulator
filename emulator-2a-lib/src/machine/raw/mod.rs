//! The actual emulated machine.

use log::{trace, warn};
#[cfg(test)]
use proptest_derive::Arbitrary;

mod signals;

use super::{
    AluInput, AluOutput, Bus, Instruction, InstructionRegister, MicroprogramRam, Register,
    RegisterNumber, Word,
};
use crate::{
    machine::MISR,
    parser::{Programsize, Stacksize},
};
pub use signals::Signals;

/// A marker for an Interrupt.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct Interrupt;

/// A waiting memory action.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct MemoryWait;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct FlagWrite;

/// State of the machine.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum State {
    /// Machine stopped regularly.
    Stopped,
    /// Machine halted after an error.
    ErrorStopped,
    /// Machine is running.
    Running,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawMachine {
    /// Keeping track of the address and content of the microprogram ram.
    microprogram_ram: MicroprogramRam,
    /// The register block. Containing the 8 registers.
    register: Register,
    /// The register containing the currently executed [`Instruction`].
    instruction_register: InstructionRegister,
    /// The state of the bus.
    bus: Bus,
    /// Do we have any pending register writes?
    pending_register_write: Option<RegisterNumber>,
    /// Do we have to update the flag register?
    pending_flag_write: Option<FlagWrite>,
    /// Do we have a pending edge interrupt?
    pending_edge_interrupt: Option<Interrupt>,
    /// Is the level interrupt alive?
    pending_level_interrupt: Option<Interrupt>,
    /// Current state of the machine.
    state: State,
    /// Do we have to wait one cycle for the memory?
    pending_wait_for_memory: Option<MemoryWait>,
    /// Latest output of the ALU
    alu_output: AluOutput,
    /// Stacksize, for stacksize supervision.
    stacksize: Stacksize,
    /// Programsize, for program counter supervision.
    programsize: Programsize,
    /// Bus content from last cycle
    last_bus_read: u8,
}

#[derive(Debug)]
#[must_use]
struct MachineAfterRegWrite<'a>(&'a mut RawMachine);

#[derive(Debug)]
#[must_use]
struct MachineAfterInstructionUpdate<'a>(&'a mut RawMachine);

#[derive(Debug)]
#[must_use]
struct MachineAfterInterruptFetching<'a>(&'a mut RawMachine);

#[derive(Debug)]
#[must_use]
struct MachineAfterWordUpdate<'a>(&'a mut RawMachine);

/// Machine after having read from the memory.
///
/// First parameter is the machine, second is the data read from the bus.
#[derive(Debug)]
#[must_use]
struct MachineAfterMemoryRead<'a>(&'a mut RawMachine);

#[derive(Debug)]
#[must_use]
struct MachineAfterAluCalculations<'a>(&'a mut RawMachine);

impl RawMachine {
    /// Create a new machine in the default state.
    ///
    /// # Examples
    /// TODO: Examples
    pub const fn new() -> Self {
        let microprogram_ram = MicroprogramRam::new();
        let register = Register::new();
        let instruction_register = InstructionRegister::new();
        let pending_register_write = None;
        let pending_flag_write = None;
        let pending_edge_interrupt = None;
        let pending_level_interrupt = None;
        let pending_wait_for_memory = None;
        let bus = Bus::new();
        let stacksize = Stacksize::default();
        let programsize = Programsize::default();
        let state = State::Running;
        let alu_output = AluOutput::default();
        let last_bus_read = 0;
        RawMachine {
            microprogram_ram,
            register,
            instruction_register,
            bus,
            pending_register_write,
            pending_flag_write,
            pending_edge_interrupt,
            pending_level_interrupt,
            state,
            pending_wait_for_memory,
            alu_output,
            stacksize,
            programsize,
            last_bus_read,
        }
    }

    /// Get a reference to the contained register block.
    pub const fn registers(&self) -> &Register {
        &self.register
    }

    /// Get a reference to the connected bus.
    pub const fn bus(&self) -> &Bus {
        &self.bus
    }

    /// State of the machine.
    pub const fn state(&self) -> State {
        self.state
    }

    /// Get a reference to the currently executed opcode instruction.
    pub const fn word(&self) -> &Instruction {
        self.instruction_register.get()
    }

    /// Get the maximum allowed stacksize, if set.
    pub const fn stacksize(&self) -> Stacksize {
        self.stacksize
    }

    /// Set the maximum allowed stacksize
    pub fn set_stacksize(&mut self, stacksize: Stacksize) {
        self.stacksize = stacksize
    }

    /// Get the maximum allowed value for the program counter (PC), the program size.
    pub const fn programsize(&self) -> Programsize {
        self.programsize
    }

    /// Set the maximum allowed program counter value.
    pub fn set_programsize(&mut self, programsize: Programsize) {
        self.programsize = programsize
    }

    /// Trigger a key edge interrupt.
    pub fn trigger_key_edge_interrupt(&mut self) {
        trace!("Key edge interrupt fired, checking control registers..");
        if self.bus.is_key_edge_int_enabled() {
            trace!("Key edge interrupt triggered successfully.");
            self.pending_edge_interrupt = Some(Interrupt);
            // TODO: I don't actually know when this needs setting. See #34
            self.bus_mut()
                .misr_mut()
                .insert(MISR::KEY_INTERRUPT_PENDING);
        }
        // TODO: I don't actually know when this needs setting. See #34
        self.bus_mut()
            .misr_mut()
            .insert(MISR::KEY_INTERRUPT_REQUEST_ACTIVE);
    }

    /// Trigger the `CONTINUE` key.
    ///
    /// This will move the state from [`State::Stopped`] -> [`State::Running`].
    pub fn trigger_key_continue(&mut self) {
        if self.state == State::Stopped {
            self.state = State::Running
        }
    }

    /// Get mutable access to the underlying bus.
    pub fn bus_mut(&mut self) -> &mut Bus {
        &mut self.bus
    }

    /// Get a reference to the underlying [`Signals`] of the machine.
    pub fn signals(&self) -> Signals<'_> {
        Signals::from(self)
    }

    /// Is the current instruction done executing?
    ///
    /// This will return `true`, iff the [`Word`] that was executed during the last
    /// clock cycle, completed the opcode [`Instruction`].
    pub const fn is_instruction_done(&self) -> bool {
        self.microprogram_ram.get_word().contains(Word::MAC3)
    }

    /// Reset the program execution.
    ///
    /// This resets:
    ///  - The program execution
    ///  - The CPU registers
    ///  - The instruction register
    ///  - The output register
    ///  - The MICR
    ///  - The UCR
    ///  - Edge interrupts
    ///  - The machine state back to Running
    pub fn cpu_reset(&mut self) {
        self.microprogram_ram.reset();
        self.register.reset();
        self.instruction_register.reset();
        self.pending_register_write = None;
        self.pending_flag_write = None;
        self.pending_edge_interrupt = None;
        self.state = State::Running;
        self.pending_wait_for_memory = None;
        self.alu_output = AluOutput::default();
        self.last_bus_read = 0;
        self.bus.cpu_reset();
    }

    /// Reset the machine.
    ///
    /// On top of the [`RawMachine::cpu_reset`], the following will be reset:
    ///  - The input register
    ///  - The raw
    ///  - The interrupt timer configuration
    ///  - Additional settings and outputs of the [`Board`][emulator-2a-lib::machine::Board]
    pub fn master_reset(&mut self) {
        self.cpu_reset();
        self.bus.master_reset();
    }

    /// Emulate a rising CLK edge.
    pub fn trigger_clock_edge(&mut self) {
        if self.state != State::Running {
            trace!("Ignoring clock. Machine halted.");
            return;
        } else if let Some(MemoryWait) = self.pending_wait_for_memory.take() {
            trace!("Skipping clock. Waiting for memory.");
            return;
        }
        trace!("");
        trace!("----- Begin of clock cycle -----");
        self.apply_pending_register_writes()
            .update_instruction_from_bus()
            .fetch_interrupts()
            .update_word()
            .read_from_memory()
            .calculate_alu_output()
            .write_to_memory();
        trace!("----- End of clock cycle -------");
        trace!("");
    }

    /// Check the stackpointer.
    pub fn is_stackpointer_valid(&self) -> bool {
        let sp = *self.register.get(RegisterNumber::R5);
        if sp >= 0xF0 {
            return false;
        }
        match self.stacksize {
            Stacksize::_0 => true,
            Stacksize::_16 => sp <= 0xD0 || sp >= 0xDF,
            Stacksize::_32 => sp <= 0xC0 || sp >= 0xCF,
            Stacksize::_48 => sp <= 0xB0 || sp >= 0xBF,
            Stacksize::_64 => sp <= 0xA0 || sp >= 0xAF,
            Stacksize::NotSet => unreachable!("BUG: The stacksize must never be UNSET"),
        }
    }

    /// Check the program counter (PC).
    pub fn is_program_counter_valid(&self) -> bool {
        let pc = *self.register.get(RegisterNumber::R3);
        if let Programsize::Size(ref n) = &self.programsize {
            // XXX: How exactly does the original compare these values?
            pc <= *n
        } else {
            panic!("BUG: The programsize cannot be AUTO or UNSET at this point");
        }
    }

    /// Writes values to the register that were created during the
    /// last cycle. This writes to the selected register if necessary
    /// and saves the flags, if requested.
    ///
    /// XXX: What should happen when the selected register is the flag register?
    fn apply_pending_register_writes(&mut self) -> MachineAfterRegWrite {
        if let Some(FlagWrite) = self.pending_flag_write.take() {
            let carry_flag = self.alu_output.carry_out();
            let zero_flag = self.alu_output.zero_out();
            let negative_flag = self.alu_output.negative_out();
            trace!(
                "Updating flags: CF: {}, ZF: {}, NF: {}",
                carry_flag as u8,
                zero_flag as u8,
                negative_flag as u8
            );
            self.register.set_carry_flag(carry_flag);
            self.register.set_zero_flag(zero_flag);
            self.register.set_negative_flag(negative_flag);
        }
        if let Some(register) = self.pending_register_write.take() {
            let new_value = self.alu_output.output();
            trace!("Updating register {:?} with {:?}", register, new_value);
            self.register.set(register, new_value);
            // Check stackpointer
            if !self.is_stackpointer_valid() {
                warn!("Stackpointer became invalid");
                self.state = State::ErrorStopped;
            }
            if !self.is_program_counter_valid() {
                warn!("Program counter became invalid");
                self.state = State::ErrorStopped;
            }
        }
        MachineAfterRegWrite(self)
    }
}

impl<'a> MachineAfterRegWrite<'a> {
    pub fn update_instruction_from_bus(self) -> MachineAfterInstructionUpdate<'a> {
        let machine = self.0;
        if machine.signals().mac1() && machine.signals().mac2() {
            // Reset the instruction register
            trace!("Resetting instruction register");
            machine.instruction_register.reset();
        } else if machine.signals().mac0() && machine.signals().mac2() {
            // Selecting next instruction
            if machine.last_bus_read == 0x00 {
                warn!("Read 0x00 instruction! Error halting");
                machine.state = State::ErrorStopped;
            } else if machine.last_bus_read == 0x01 {
                warn!("Read 0x01 instruction. Halting.");
                machine.state = State::Stopped;
            } else if machine.last_bus_read == 0b0010_1100 {
                // We need to clear some MISR flags once the program returns from interrupt
                trace!("RETI detected. Removing MISR flags");
                // TODO: I don't actually know when this needs setting. See #34
                machine
                    .bus_mut()
                    .misr_mut()
                    .remove(MISR::KEY_INTERRUPT_PENDING);
                machine
                    .bus_mut()
                    .misr_mut()
                    .remove(MISR::KEY_INTERRUPT_REQUEST_ACTIVE);
            }
            machine.instruction_register.set_raw(machine.last_bus_read);
            trace!("Next instruction: {:?}", machine.instruction_register);
        }
        MachineAfterInstructionUpdate(machine)
    }
}

impl<'a> MachineAfterInstructionUpdate<'a> {
    /// TODO: Interrupts need verification!
    pub fn fetch_interrupts(self) -> MachineAfterInterruptFetching<'a> {
        let machine = self.0;
        trace!("Fetching interrupts");
        machine.pending_edge_interrupt = machine
            .pending_edge_interrupt
            .take()
            .or_else(|| machine.bus.take_edge_interrupt());
        machine.pending_level_interrupt = machine
            .pending_level_interrupt
            .take()
            .or_else(|| machine.bus.get_level_interrupt());
        MachineAfterInterruptFetching(machine)
    }
}

impl<'a> MachineAfterInterruptFetching<'a> {
    pub fn update_word(self) -> MachineAfterWordUpdate<'a> {
        let machine = self.0;
        let next_mp_ram_addr = machine.signals().next_microprogram_address();
        trace!("New word: {:?}", machine.microprogram_ram.get_word());
        // Clearing edge interrupt if used
        if machine.signals().interrupt_logic_1() {
            trace!("Clearing edge interrupt");
            machine.pending_edge_interrupt = None;
        }
        // Do not update MP_RAM address before interrupt clear check
        trace!("New MP_RAM address: {}", next_mp_ram_addr);
        machine.microprogram_ram.set_address(next_mp_ram_addr);
        MachineAfterWordUpdate(machine)
    }
}

impl<'a> MachineAfterWordUpdate<'a> {
    pub fn read_from_memory(self) -> MachineAfterMemoryRead<'a> {
        let machine = self.0;
        // Get A-output of register block
        let selected_reg_a = machine.signals().selected_register_a();
        let register_out_a = machine.register.get(selected_reg_a);
        if machine.signals().busen() {
            machine.last_bus_read = machine.bus.read(*register_out_a);
            trace!(
                "Reading {:?} from bus address {:?}",
                machine.last_bus_read,
                *register_out_a
            );
            if *register_out_a <= 0xEF {
                trace!("Generating artificial wait signal");
                machine.pending_wait_for_memory = Some(MemoryWait);
            }
        } else {
            machine.last_bus_read = 0;
        }
        MachineAfterMemoryRead(machine)
    }
}

impl<'a> MachineAfterMemoryRead<'a> {
    pub fn calculate_alu_output(self) -> MachineAfterAluCalculations<'a> {
        let machine = self.0;
        // Calculate inputs to the alu
        let alu_input_a = if machine.signals().maluia() {
            machine.last_bus_read
        } else {
            let selected_reg_a = machine.signals().selected_register_a();
            *machine.register.get(selected_reg_a)
        };
        let alu_input_b = if machine.signals().maluib() {
            machine.signals().alu_input_b_constant()
        } else {
            let selected_reg_b = machine.signals().selected_register_b();
            *machine.register.get(selected_reg_b)
        };
        // Actually calculate the alu output
        let alu_input = AluInput::new(alu_input_a, alu_input_b, machine.signals().carry_flag());
        trace!("ALU Input : {:?}", alu_input);
        trace!("ALU Fn    : {:?}", machine.signals().alu_select());
        machine.alu_output = AluOutput::from_input(&alu_input, &machine.signals().alu_select());
        trace!("ALU Output: {:?}", machine.alu_output);
        // Update registers if necessary
        if machine.signals().mrgwe() {
            let selected_register = machine.signals().selected_register_for_writing();
            machine.pending_register_write = Some(selected_register);
        }
        if machine.signals().mchflg() {
            machine.pending_flag_write = Some(FlagWrite);
        }
        MachineAfterAluCalculations(machine)
    }
}

impl<'a> MachineAfterAluCalculations<'a> {
    pub fn write_to_memory(self) {
        let machine = self.0;
        if machine.signals().buswr() {
            let selected_reg_a = machine.signals().selected_register_a();
            let register_out_a = machine.register.get(selected_reg_a);
            trace!(
                "Writing {} to bus address {}",
                machine.alu_output.output(),
                register_out_a
            );
            machine
                .bus
                .write(*register_out_a, machine.alu_output.output());
            if *register_out_a <= 0xEF {
                trace!("Generating artificial wait signal");
                machine.pending_wait_for_memory = Some(MemoryWait);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    impl RawMachine {
        prop_compose! {
            pub fn arbitrary()(
                microprogram_ram in any::<MicroprogramRam>(),
                register in any::<Register>(),
                instruction_register in any::<InstructionRegister>(),
                bus in Bus::arbitrary(),
                pending_register_write in any::<Option<RegisterNumber>>(),
                pending_flag_write in any::<Option<FlagWrite>>(),
                pending_edge_interrupt in any::<Option<Interrupt>>(),
                pending_level_interrupt in any::<Option<Interrupt>>(),
                state in any::<State>(),
                pending_wait_for_memory in any::<Option<MemoryWait>>(),
                alu_output in any::<AluOutput>(),
                stacksize in any::<Stacksize>(),
                programsize in any::<Programsize>(),
                last_bus_read in any::<u8>(),
            ) -> Self {
                RawMachine {
                    microprogram_ram,
                    register,
                    instruction_register,
                    bus,
                    pending_register_write,
                    pending_flag_write,
                    pending_edge_interrupt,
                    pending_level_interrupt,
                    state,
                    pending_wait_for_memory,
                    alu_output,
                    stacksize,
                    programsize,
                    last_bus_read
                }
            }
        }
    }

    proptest! {
        #[test]
        fn pending_register_write_is_cleared_during_reset(mut machine in RawMachine::arbitrary()) {
            let pristine = machine.clone();
            machine.cpu_reset();
            assert_eq!(
                machine.pending_register_write,
                RawMachine::new().pending_register_write
            );
            // Let's make sure it works for the master reset aswell
            machine = pristine;
            machine.master_reset();
            assert_eq!(
                machine.pending_register_write,
                RawMachine::new().pending_register_write
            );
        }

        #[test]
        fn pending_flag_write_is_cleared_during_reset(mut machine in RawMachine::arbitrary()) {
            machine.cpu_reset();
            assert_eq!(
                machine.pending_flag_write,
                RawMachine::new().pending_flag_write
            );
        }

        #[test]
        fn pending_edge_interrupt_is_cleared_during_reset(mut machine in RawMachine::arbitrary()) {
            machine.cpu_reset();
            assert_eq!(
                machine.pending_edge_interrupt,
                RawMachine::new().pending_edge_interrupt
            );
        }

        #[test]
        fn pending_wait_for_memory_is_cleared_during_reset(mut machine in RawMachine::arbitrary()) {
            machine.cpu_reset();
            assert_eq!(
                machine.pending_wait_for_memory,
                RawMachine::new().pending_wait_for_memory
            );
        }

        #[test]
        fn state_is_reset_correctly(mut machine in RawMachine::arbitrary()) {
            machine.cpu_reset();
            assert_eq!(
                machine.state,
                RawMachine::new().state
            );
        }

        #[test]
        fn alu_output_is_reset_correctly(mut machine in RawMachine::arbitrary()) {
            machine.cpu_reset();
            assert_eq!(
                machine.alu_output,
                RawMachine::new().alu_output
            );
        }

        #[test]
        fn last_bus_read_is_reset_correctly(mut machine in RawMachine::arbitrary()) {
            machine.cpu_reset();
            assert_eq!(
                machine.last_bus_read,
                RawMachine::new().last_bus_read
            );
        }

        #[test]
        fn stacksize_is_never_reset(mut machine in RawMachine::arbitrary()) {
            let pristine = machine.clone();
            machine.cpu_reset();
            assert_eq!(machine.stacksize, pristine.stacksize);
            machine.master_reset();
            assert_eq!(machine.stacksize, pristine.stacksize);
        }
    }
}
