//! The actual emulated machine.

use log::trace;
use parser2a::asm::Stacksize;

mod alu;
mod board;
mod bus;
mod instruction;
mod mp_ram;
mod register;
mod signal;

pub use alu::Alu;
pub use board::{Board, DASR, DAISR, DAICR};
pub use bus::Bus;
pub use instruction::Instruction;
pub use mp_ram::{MP28BitWord, MicroprogramRam};
pub use register::{Register, RegisterNumber};
pub use signal::Signal;

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
    mp_ram: MicroprogramRam,
    /// The register block.
    reg: Register,
    /// The currently executed instruction byte.
    current_instruction: Instruction,
    /// The state of the bus.
    pub(crate) bus: Bus,
    /// The pending register write from last iteration.
    pending_register_write: Option<(RegisterNumber, u8)>,
    /// The pending flag write from last iteration.
    pending_flag_write: Option<(bool, bool, bool)>,
    edge_int: bool,
    level_int: bool,
    /// Lines of the program.
    program_lines: Vec<(String, usize)>,
    state: State,
    /// Waiting on rising clock edge for the memory.
    waiting_for_memory: bool,
    /// Is the last instruction done?
    instruction_done: bool,
    /// Counting clock edges.
    clk_counter: usize,
    /// Counting drawing cycles for conditional drawings.
    draw_counter: usize,
    /// Stacksize, if any program is loaded.
    /// For error checking.
    stacksize: Option<Stacksize>,
}

impl Machine {
    /// Create and run a new Minirechner 2a with an optional program.
    pub fn new() -> Self {
        let mp_ram = MicroprogramRam::new();
        let reg = Register::new();
        let bus = Bus::new();
        let current_instruction = Instruction::reset();
        let pending_register_write = None;
        let pending_flag_write = None;
        let edge_int = false;
        let level_int = false;
        let program_lines = vec![];
        let waiting_for_memory = false;
        let instruction_done = false;
        let clk_counter = 0;
        let draw_counter = 0;
        let stacksize = None;
        let state = State::Running;
        let machine = Machine {
            mp_ram,
            reg,
            bus,
            current_instruction,
            state,
            pending_register_write,
            pending_flag_write,
            edge_int,
            level_int,
            program_lines,
            waiting_for_memory,
            instruction_done,
            clk_counter,
            draw_counter,
            stacksize,
        };
        machine
    }
    /// Input `number` into input register `FC`.
    pub fn input_fc(&mut self, number: u8) {
        self.bus.input_fc(number)
    }
    /// Input `number` into input register `FD`.
    pub fn input_fd(&mut self, number: u8) {
        self.bus.input_fd(number)
    }
    /// Input `number` into input register `FE`.
    pub fn input_fe(&mut self, number: u8) {
        self.bus.input_fe(number)
    }
    /// Input `number` into input register `FF`.
    pub fn input_ff(&mut self, number: u8) {
        self.bus.input_ff(number)
    }
    /// Content of output register `FE`.
    pub fn output_fe(&self) -> u8 {
        self.bus.output_fe()
    }
    /// Content of output register `FF`.
    pub fn output_ff(&self) -> u8 {
        self.bus.output_ff()
    }
    /// Next clock rising edge.
    pub fn clk(&mut self) {
        // Increase the counter
        self.clk_counter = self.clk_counter.overflowing_add(1).0;
        // Execute the update
        self.update()
    }
    /// Is the current instruction done executing?
    pub fn is_instruction_done(&self) -> bool {
        self.instruction_done
    }
    /// State of the machine.
    pub fn state(&self) -> State {
        self.state
    }
    /// Continue the machine after a stop.
    pub fn continue_from_stop(&mut self) {
        if self.state == State::Stopped {
            self.state = State::Running;
        }
    }
    /// Get read access to the register block.
    pub fn registers(&self) -> &Register {
        &self.reg
    }
    /// Get read access to the bus.
    pub fn bus(&self) -> &Bus {
        &self.bus
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
        self.reg.reset();
        self.bus.reset();
        self.pending_flag_write = None;
        self.pending_register_write = None;
        self.current_instruction = Instruction::reset();
        self.edge_int = false;
        self.level_int = false;
        self.mp_ram.reset();
        self.waiting_for_memory = false;
        self.state = State::Running;
    }
    /// Input a key edge interrupt.
    pub fn key_edge_int(&mut self) {
        trace!("Received key edge interrupt");
        if self.bus.is_key_edge_int_enabled() {
            self.edge_int = true;
        }
    }
    /// Is key edge interrupt enabled?
    pub fn is_key_edge_int_enabled(&self) -> bool {
        self.bus.is_key_edge_int_enabled()
    }
    /// Update the machine.
    /// This should be equivalent to a CLK signal on the real machine.
    fn update(&mut self) {
        if self.state != State::Running {
            return;
        } else if self.waiting_for_memory {
            self.waiting_for_memory = false;
            return;
        }
        // ------------------------------------------------------------
        // Update register block with values from last iteration
        // ------------------------------------------------------------
        if let Some((r, value)) = self.pending_register_write {
            trace!("Setting register: {:?} = {:>02X}", r, value);
            self.pending_register_write = None;
            self.reg.set(r, value);
            // Check stackpointer
            if let Some(stacksize) = self.stacksize {
                if !self.reg.is_stackpointer_valid(stacksize) {
                    self.state = State::ErrorStopped;
                }
            }
        }
        if let Some((co, zo, no)) = self.pending_flag_write {
            trace!("Updating flags: CO: {}, ZO: {}, NO: {}", co, zo, no);
            self.reg.update_co(co);
            self.reg.update_zo(zo);
            self.reg.update_no(no);
        }
        // ------------------------------------------------------------
        // Use microprogram word from last iteration
        // ------------------------------------------------------------
        let mp_ram_out = self.mp_ram.get();
        // Safe MAC3 for later
        self.instruction_done = mp_ram_out.contains(MP28BitWord::MAC3);
        let mut sig = Signal::new(&mp_ram_out, &self.current_instruction);
        trace!("Address: {:>08b} ({0})", self.mp_ram.current_addr());
        trace!("Word: {:?}", mp_ram_out);
        // ------------------------------------------------------------
        // Add some interrupts, if necessary
        // ------------------------------------------------------------
        self.edge_int |= self.bus.fetch_edge_int();
        self.level_int = self.bus.has_level_int();
        // ------------------------------------------------------------
        // Add inputs to sig
        // ------------------------------------------------------------
        sig.set_edge_int(self.edge_int);
        sig.set_level_int(self.level_int);
        // ------------------------------------------------------------
        // Get outputs of register block
        // ------------------------------------------------------------
        sig.set_cf(self.reg.cf());
        sig.set_zf(self.reg.zf());
        sig.set_nf(self.reg.nf());
        sig.set_ief(self.reg.ief());
        let doa = self.reg.doa(&sig);
        let dob = self.reg.dob(&sig);
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
                self.waiting_for_memory = true;
                trace!("Generating wait signal");
            }
        }
        // ------------------------------------------------------------
        // Update current instruction from bus
        // ------------------------------------------------------------
        if sig.mac1() && sig.mac2() {
            // Reset the instruction register
            trace!("Resetting instruction register");
            self.current_instruction = Instruction::reset();
            sig.set_current_instruction(&self.current_instruction);
        } else if sig.mac0() && sig.mac2() {
            // Selecting next instruction
            self.current_instruction = Instruction::from_bits_truncate(bus_content);
            if bus_content == 0x00 {
                self.state = State::ErrorStopped;
            } else if bus_content == 0x01 {
                self.state = State::Stopped;
            }
            sig.set_current_instruction(&self.current_instruction);
            trace!("Instruction: {:?}", self.current_instruction);
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
        let alu_fn = (sig.malus3(), sig.malus2(), sig.malus1(), sig.malus0()).into();
        trace!("ALU fn: {:?}", alu_fn);
        let memdo = Alu::output(sig.cf(), alu_in_a, alu_in_b, alu_fn);
        let co = Alu::co(sig.cf(), alu_in_a, alu_in_b, alu_fn);
        let zo = Alu::zo(sig.cf(), alu_in_a, alu_in_b, alu_fn);
        let no = Alu::no(sig.cf(), alu_in_a, alu_in_b, alu_fn);
        trace!("MEMDO: 0x{:>02X}", memdo);
        // ------------------------------------------------------------
        // Add ALU outputs to signals
        // ------------------------------------------------------------
        sig.set_co(co);
        sig.set_zo(zo);
        sig.set_no(no);
        trace!("CO: {}, ZO: {}, NO: {}", co, zo, no);
        // Update registers if necessary
        if sig.mrgwe() {
            let selected_reg = Register::get_selected(&sig);
            self.pending_register_write = Some((selected_reg, memdo));
            trace!("New pending register write");
        }
        if sig.mchflg() {
            self.pending_flag_write = Some((sig.co(), sig.zo(), sig.no()));
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
                self.waiting_for_memory = true;
            }
        }
        // Clearing edge interrupt if used
        if self.edge_int && sig.na0() && sig.mac0() && sig.mac1() {
            trace!("Clearing edge interrupt");
            self.edge_int = false;
        }
        // ------------------------------------------------------------
        // Select next microprogram word
        // ------------------------------------------------------------
        let next_mp_ram_addr = MicroprogramRam::get_addr(&sig);
        trace!("Next MP_RAM address: {}", next_mp_ram_addr);
        self.mp_ram.select(next_mp_ram_addr);
    }
}
