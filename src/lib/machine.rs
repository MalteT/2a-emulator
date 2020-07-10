//! The actual emulated machine.

use log::trace;
use parser2a::asm::Stacksize;

use crate::{
    AluInput, AluOutput, Bus, Instruction, InstructionRegister, MicroprogramRam, Register,
    RegisterNumber, Signal, Word,
};

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
    /// Input a key edge interrupt.
    pub fn key_edge_int(&mut self) {
        trace!("Received key edge interrupt");
        if self.bus.is_key_edge_int_enabled() {
            self.pending_edge_interrupt = Some(Interrupt);
        }
    }
    /// Emulate a rising CLK edge.
    pub fn trigger_clock_edge(&mut self) {
        if self.state != State::Running {
            return;
        } else if let Some(MemoryAccess) = self.pending_memory_access.take() {
            return;
        }
        // ------------------------------------------------------------
        // Update register block with values from last iteration
        // ------------------------------------------------------------
        if let Some(register) = self.pending_register_write.take() {
            self.register.set(register, self.alu_output.output());
            // Check stackpointer
            if let Some(stacksize) = self.stacksize {
                if !self.register.is_stackpointer_valid(stacksize) {
                    self.state = State::ErrorStopped;
                }
            }
        }
        if let Some(FlagWrite) = self.pending_flag_write.take() {
            self.register.set_carry_flag(self.alu_output.carry_out());
            self.register.set_zero_flag(self.alu_output.zero_out());
            self.register
                .set_negative_flag(self.alu_output.negative_out());
        }
        // ------------------------------------------------------------
        // Use microprogram word from last iteration
        // ------------------------------------------------------------
        let mp_ram_out = self.microprogram_ram.get_word();
        // Safe MAC3 for later
        self.is_instruction_done = mp_ram_out.contains(Word::MAC3);
        let mut sig = Signal::new();
        sig = sig
            .set_instruction(*self.instruction_register.get())
            .set_word(*mp_ram_out);
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
        sig = sig.set_flags(self.register.flags());
        let doa = self.register.doa(&sig);
        let dob = self.register.dob(&sig);
        trace!("DOA: {:>02X}", doa);
        trace!("DOB: {:>02X}", dob);
        // ------------------------------------------------------------
        // Read value from bus at selected address
        // ------------------------------------------------------------
        let mut bus_content = 0;
        if sig.busen() {
            bus_content = self.bus.read(doa);
            trace!("MEMDI: 0x{:>02X}", bus_content);
            if doa <= 0xEF {
                trace!("Generating wait signal");
                self.pending_memory_access = Some(MemoryAccess);
            }
        }
        // ------------------------------------------------------------
        // Update current instruction from bus
        // ------------------------------------------------------------
        if sig.mac1() && sig.mac2() {
            // Reset the instruction register
            trace!("Resetting instruction register");
            self.instruction_register.reset();
            sig = sig.set_instruction(*self.instruction_register.get());
        } else if sig.mac0() && sig.mac2() {
            // Selecting next instruction
            if bus_content == 0x00 {
                self.state = State::ErrorStopped;
            } else if bus_content == 0x01 {
                self.state = State::Stopped;
            }
            self.instruction_register.set_raw(bus_content);
            sig = sig.set_instruction(*self.instruction_register.get());
        }
        // ------------------------------------------------------------
        // Calculate the ALU output
        // ------------------------------------------------------------
        let alu_in_a = if sig.maluia() { bus_content } else { doa };
        let alu_in_b = if sig.maluib() {
            if sig.mrgab3() {
                0b1111_1000
                    + ((sig.mrgab2() as u8) << 2)
                    + ((sig.mrgab1() as u8) << 1)
                    + sig.mrgab0() as u8
            } else {
                ((sig.mrgab2() as u8) << 2) + ((sig.mrgab1() as u8) << 1) + sig.mrgab0() as u8
            }
        } else {
            dob
        };
        let alu_in = AluInput::new(alu_in_a, alu_in_b, sig.carry_out());
        self.alu_output = AluOutput::from_input(&alu_in, &sig.alu_select());
        let memdo = self.alu_output.output();
        let co = self.alu_output.carry_out();
        let zo = self.alu_output.zero_out();
        let no = self.alu_output.negative_out();
        trace!("MEMDO: 0x{:>02X}", memdo);
        // ------------------------------------------------------------
        // Add ALU outputs to signals
        // ------------------------------------------------------------
        sig = sig.set_carry_out(co).set_zero_out(zo).set_negative_out(no);
        trace!("CO: {}, ZO: {}, NO: {}", co, zo, no);
        // Update registers if necessary
        if sig.mrgwe() {
            let selected_register = Register::get_selected(&sig);
            self.pending_register_write = Some(selected_register);
            trace!("New pending register write");
        }
        if sig.mchflg() {
            self.pending_flag_write = Some(FlagWrite);
            trace!("New pending flag write");
        }
        // ------------------------------------------------------------
        // Update bus
        // TODO: wait
        // ------------------------------------------------------------
        if sig.buswr() {
            trace!("Writing 0x{:>02X} to bus address 0x{:>02X}", memdo, doa);
            self.bus.write(doa, memdo);
            if doa <= 0xEF {
                trace!("Generating wait signal");
                self.pending_memory_access = Some(MemoryAccess);
            }
        }
        // ------------------------------------------------------------
        // Select next microprogram word
        // ------------------------------------------------------------
        let next_mp_ram_addr = MicroprogramRam::get_addr(
            &sig,
            self.pending_edge_interrupt.is_some(),
            self.pending_level_interrupt.is_some(),
        );
        trace!("Next MP_RAM address: {}", next_mp_ram_addr);
        self.microprogram_ram.set_address(next_mp_ram_addr);
        // Clearing edge interrupt if used
        if self.pending_edge_interrupt.is_some() && sig.na0() && sig.mac0() && sig.mac1() {
            trace!("Clearing edge interrupt");
            self.pending_edge_interrupt = None;
        }
    }
}
