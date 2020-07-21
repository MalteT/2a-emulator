//! The actual emulated machine.

use log::{trace, warn};
use parser2a::asm::Stacksize;

mod signals;

use crate::{
    AluInput, AluOutput, Bus, Instruction, InstructionRegister, MicroprogramRam, Register,
    RegisterNumber, Word,
};
pub use signals::Signals;

/// A marker for an Interrupt.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Interrupt;

/// A marker for memory access.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MemoryAccess;

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
pub struct Machine {
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
    pending_memory_access: Option<MemoryAccess>,
    /// Latest output of the ALU
    alu_output: AluOutput,
    /// Stacksize, for stacksize supervision.
    stacksize: Option<Stacksize>,
    /// Did the last word finish the assembly instruction?
    is_instruction_done: bool,
}

#[derive(Debug)]
#[must_use]
struct MachineAfterRegWrite<'a>(&'a mut Machine);

impl Machine {
    /// Create a new machine in the default state.
    ///
    /// # Examples
    ///
    /// ```
    /// # use emulator_2a_lib::Machine;
    /// let mut machine = Machine::new();
    pub const fn new() -> Self {
        let microprogram_ram = MicroprogramRam::new();
        let register = Register::new();
        let instruction_register = InstructionRegister::new();
        let pending_register_write = None;
        let pending_flag_write = None;
        let pending_edge_interrupt = None;
        let pending_level_interrupt = None;
        let pending_memory_access = None;
        let is_instruction_done = false;
        let bus = Bus::new();
        let stacksize = None;
        let state = State::Running;
        let alu_output = AluOutput::default();
        Machine {
            microprogram_ram,
            register,
            instruction_register,
            pending_memory_access,
            is_instruction_done,
            alu_output,
            bus,
            state,
            pending_register_write,
            pending_flag_write,
            stacksize,
            pending_edge_interrupt,
            pending_level_interrupt,
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
    pub const fn stacksize(&self) -> Option<Stacksize> {
        self.stacksize
    }

    /// Set the maximum allowed stacksize
    pub fn set_stacksize(&mut self, stacksize: Stacksize) {
        self.stacksize = Some(stacksize)
    }

    /// Trigger a key edge interrupt.
    pub fn trigger_key_edge_interrupt(&mut self) {
        if self.bus.is_key_edge_int_enabled() {
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
    pub fn signals<'a>(&'a self) -> Signals<'a> {
        Signals::from(self)
    }

    /// Is the current instruction done executing?
    ///
    /// This will return `true`, iff the [`Word`] that was executed during the last
    /// clock cycle, completed the opcode [`Instruction`].
    pub const fn is_instruction_done(&self) -> bool {
        self.is_instruction_done
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
        self.register.reset();
        self.bus.reset();
        self.pending_flag_write = None;
        self.pending_register_write = None;
        self.microprogram_ram.reset();
        self.pending_memory_access = None;
        self.state = State::Running;
    }

    /// Emulate a rising CLK edge.
    pub fn trigger_clock_edge(&mut self) {
        if self.state != State::Running {
            return;
        } else if let Some(MemoryAccess) = self.pending_memory_access.take() {
            return;
        }
        self.apply_pending_register_writes();
        // ------------------------------------------------------------
        // Use microprogram word from last iteration
        // ------------------------------------------------------------
        let mp_ram_out = self.microprogram_ram.get_word();
        // Safe MAC3 for later
        self.is_instruction_done = mp_ram_out.contains(Word::MAC3);
        trace!(
            "Address: {:>08b} ({0})",
            self.microprogram_ram.get_address()
        );
        trace!("Word: {:?}", mp_ram_out);
        // ------------------------------------------------------------
        // Add some interrupts, if necessary
        // ------------------------------------------------------------
        self.pending_edge_interrupt = self
            .pending_edge_interrupt
            .take()
            .or(self.bus.take_edge_interrupt());
        self.pending_level_interrupt = self
            .pending_level_interrupt
            .take()
            .or(self.bus.get_level_interrupt());
        // ------------------------------------------------------------
        // Add inputs to sig
        // ------------------------------------------------------------
        //sig.set_edge_int(self.edge_int);
        //sig.set_level_int(self.level_int);
        // ------------------------------------------------------------
        // Get outputs of register block
        // ------------------------------------------------------------
        let selected_reg_a = self.signals().selected_register_a();
        let selected_reg_b = self.signals().selected_register_b();
        let register_out_a = self.register.get(selected_reg_a);
        let register_out_b = self.register.get(selected_reg_b);
        trace!("DOA: {:>02X}", register_out_a);
        trace!("DOB: {:>02X}", register_out_a);
        // ------------------------------------------------------------
        // Read value from bus at selected address
        // ------------------------------------------------------------
        let mut bus_content = 0;
        if self.signals().busen() {
            bus_content = self.bus.read(*register_out_a);
            trace!("MEMDI: 0x{:>02X}", bus_content);
            if *register_out_a <= 0xEF {
                trace!("Generating wait signal");
                self.pending_memory_access = Some(MemoryAccess);
            }
        }
        // ------------------------------------------------------------
        // Update current instruction from bus
        // ------------------------------------------------------------
        if self.signals().mac1() && self.signals().mac2() {
            // Reset the instruction register
            trace!("Resetting instruction register");
            self.instruction_register.reset();
        } else if self.signals().mac0() && self.signals().mac2() {
            // Selecting next instruction
            if bus_content == 0x00 {
                self.state = State::ErrorStopped;
            } else if bus_content == 0x01 {
                self.state = State::Stopped;
            }
            self.instruction_register.set_raw(bus_content);
        }
        // ------------------------------------------------------------
        // Calculate the ALU output
        // ------------------------------------------------------------
        let alu_input_a = if self.signals().maluia() {
            bus_content
        } else {
            *register_out_a
        };
        let alu_input_b = if self.signals().maluib() {
            self.signals().alu_input_b_constant()
        } else {
            *register_out_b
        };
        let alu_input = AluInput::new(alu_input_a, alu_input_b, self.signals().carry_out());
        self.alu_output = AluOutput::from_input(&alu_input, &self.signals().alu_select());
        let memdo = self.alu_output.output();
        trace!("MEMDO: 0x{:>02X}", memdo);
        // ------------------------------------------------------------
        // Add ALU outputs to signals
        // ------------------------------------------------------------
        // Update registers if necessary
        if self.signals().mrgwe() {
            trace!("New pending register write");
            let selected_register = self.signals().selected_register_for_writing();
            self.pending_register_write = Some(selected_register);
        }
        if self.signals().mchflg() {
            self.pending_flag_write = Some(FlagWrite);
            trace!("New pending flag write");
        }
        // ------------------------------------------------------------
        // Update bus
        // ------------------------------------------------------------
        if self.signals().buswr() {
            trace!(
                "Writing 0x{:>02X} to bus address 0x{:>02X}",
                memdo,
                register_out_a
            );
            self.bus.write(*register_out_a, memdo);
            if *register_out_a <= 0xEF {
                trace!("Generating wait signal");
                self.pending_memory_access = Some(MemoryAccess);
            }
        }
        // ------------------------------------------------------------
        // Select next microprogram word
        // ------------------------------------------------------------
        let next_mp_ram_addr = self.signals().next_microprogram_address();
        trace!("Next MP_RAM address: {}", next_mp_ram_addr);
        self.microprogram_ram.set_address(next_mp_ram_addr);
        // Clearing edge interrupt if used
        if self.signals().interrupt_logic_3() {
            trace!("Clearing edge interrupt");
            self.pending_edge_interrupt = None;
        }
    }
    /// Check the stackpointer.
    pub fn is_stackpointer_valid(&self) -> bool {
        let sp = *self.register.get(RegisterNumber::R4);
        if sp >= 0xF0 {
            return false;
        }
        if let Some(stacksize) = self.stacksize {
            match stacksize {
                Stacksize::_16 => sp <= 0xD0 || sp >= 0xDF,
                Stacksize::_32 => sp <= 0xC0 || sp >= 0xCF,
                Stacksize::_48 => sp <= 0xB0 || sp >= 0xBF,
                Stacksize::_64 => sp <= 0xA0 || sp >= 0xAF,
                Stacksize::NotSet => true,
            }
        } else {
            true
        }
    }
    /// Writes values to the register that were created during the
    /// last cycle. This writes to the selected register if necessary
    /// and saves the flags, if requested.
    ///
    /// XXX: What should happen when the selected register is the flag register?
    fn apply_pending_register_writes(&mut self) -> MachineAfterRegWrite {
        if let Some(register) = self.pending_register_write.take() {
            self.register.set(register, self.alu_output.output());
            // Check stackpointer
            if !self.is_stackpointer_valid() {
                warn!("Stackpointer became invalid");
                self.state = State::ErrorStopped;
            }
        }
        if let Some(FlagWrite) = self.pending_flag_write.take() {
            self.register.set_carry_flag(self.alu_output.carry_out());
            self.register.set_zero_flag(self.alu_output.zero_out());
            self.register
                .set_negative_flag(self.alu_output.negative_out());
        }
        MachineAfterRegWrite(self)
    }
}

impl<'a> MachineAfterRegWrite<'a> {

}
