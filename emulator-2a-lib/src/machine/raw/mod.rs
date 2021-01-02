//! The actual emulated machine.

use log::{trace, warn};

mod signals;

use super::{
    AluInput, AluOutput, Bus, Instruction, InstructionRegister, MicroprogramRam, Register,
    RegisterNumber, Word,
};
use crate::parser::Stacksize;
pub use signals::Signals;

/// A marker for an Interrupt.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Interrupt;

/// A waiting memory action.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MemoryWait;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FlagWrite;

/// State of the machine.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum State {
    /// Machine stopped regularly.
    Stopped,
    /// Machine halted after an error.
    ErrorStopped,
    /// Machine is running.
    Running,
}

#[derive(Debug)]
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
        let stacksize = Stacksize::_16;
        let state = State::Running;
        let alu_output = AluOutput::default();
        let last_bus_read = 0;
        RawMachine {
            microprogram_ram,
            register,
            instruction_register,
            pending_wait_for_memory,
            alu_output,
            bus,
            state,
            pending_register_write,
            pending_flag_write,
            stacksize,
            pending_edge_interrupt,
            pending_level_interrupt,
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

    /// Trigger a key edge interrupt.
    pub fn trigger_key_edge_interrupt(&mut self) {
        if self.bus.is_key_edge_int_enabled() {
            trace!("Key edge interrupt triggered.");
            self.pending_edge_interrupt = Some(Interrupt);
        }
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

    /// Reset the machine.
    ///
    /// - Clear all registers.
    /// - Load the default instruction into the instruction register.
    /// - Clear microprogram ram outputs.
    ///
    /// It does *not*:
    /// - Delete the memory (or anything on the bus).
    // # TODO: Do we need to reset interrupt inputs?
    pub fn reset(&mut self) {
        self.microprogram_ram.reset();
        self.register.reset();
        self.instruction_register.reset();
        self.bus.reset();
        self.pending_register_write = None;
        self.pending_flag_write = None;
        self.state = State::Running;
        self.pending_wait_for_memory = None;
        self.alu_output = AluOutput::default();
        self.last_bus_read = 0;
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
            Stacksize::_16 => sp <= 0xD0 || sp >= 0xDF,
            Stacksize::_32 => sp <= 0xC0 || sp >= 0xCF,
            Stacksize::_48 => sp <= 0xB0 || sp >= 0xBF,
            Stacksize::_64 => sp <= 0xA0 || sp >= 0xAF,
            Stacksize::NotSet => true,
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
        trace!("New MP_RAM address: {}", next_mp_ram_addr);
        machine.microprogram_ram.set_address(next_mp_ram_addr);
        trace!("New word: {:?}", machine.microprogram_ram.get_word());
        // Clearing edge interrupt if used
        if machine.signals().interrupt_logic_3() {
            trace!("Clearing edge interrupt");
            machine.pending_edge_interrupt = None;
        }
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
