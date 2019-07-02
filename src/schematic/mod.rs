use log::trace;
use mr2a_asm_parser::asm::{Asm, Register as RegNum};
use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
use tui::widgets::Widget;

use std::time::Instant;

mod alu;
mod bus;
mod fns;
mod instruction;
mod mp_ram;
mod register;
mod signal;

pub use alu::Alu;
pub use bus::Bus;
pub use fns::*;
pub use instruction::Instruction;
pub use mp_ram::{MP28BitWord, MicroprogramRam};
pub use register::Register;
pub use signal::Signal;

use crate::compiler::Translator;
use crate::tui::display::Display;

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Ord, Eq)]
pub enum Part {
    Al1,
    Al2,
    Al3,
    Il1,
    Il2,
    Iff1,
    Iff2,
    InterruptLogic,
}

#[derive(Debug)]
pub struct Machine {
    mp_ram: MicroprogramRam,
    /// The register block.
    reg: Register,
    /// The currently executed instruction byte.
    current_instruction: Instruction,
    /// The state of the bus.
    bus: Bus,
    /// The pending register write from last iteration.
    pending_register_write: Option<(RegisterNumber, u8)>,
    /// The pending flag write from last iteration.
    pending_flag_write: Option<(bool, bool, bool)>,
    input_edge_int: bool,
    input_level_int: bool,
    /// Frequency measurements
    measured_frequency: f32,
    frequency_measure_last_measurement: Instant,
}

impl Machine {
    /// Create a new Minirechner 2a
    pub fn new() -> Self {
        let mp_ram = MicroprogramRam::new();
        let reg = Register::new();
        let bus = Bus::new();
        let current_instruction = Instruction::reset();
        let pending_register_write = None;
        let pending_flag_write = None;
        let input_edge_int = false;
        let input_level_int = false;
        let measured_frequency = 1000.0;
        let frequency_measure_last_measurement = Instant::now();
        Machine {
            mp_ram,
            reg,
            bus,
            current_instruction,
            pending_register_write,
            pending_flag_write,
            input_edge_int,
            input_level_int,
            measured_frequency,
            frequency_measure_last_measurement,
        }
    }
    /// Run the given [`Asm`] program.
    pub fn run(&mut self, program: &Asm) {
        let bytecode = Translator::compile(program);
        let mut address = 0;
        for byte in bytecode.bytes() {
            self.bus.write(address, *byte);
            address += 1;
        }
    }
    /// TODO: Dummy
    pub fn clk(&mut self) {
        self.update()
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
        self.current_instruction = Instruction::reset();
        self.mp_ram.reset();
    }
    /// Input an edge interrupt.
    pub fn edge_int(&mut self) {
        trace!("MACHINE: Received edge interrupt");
        self.input_edge_int = true;
    }
    /// TODO: Dummy
    pub fn show(&mut self, _part: Part) {}
    /// Update the machine.
    /// This should be equivalent to a CLK signal on the real machine.
    fn update(&mut self) {
        let diff = Instant::now() - self.frequency_measure_last_measurement;
        self.measured_frequency = 1_000_000_000.0 / diff.as_nanos() as f32;
        self.frequency_measure_last_measurement = Instant::now();
        // ------------------------------------------------------------
        // Update register block with values from last iteration
        // ------------------------------------------------------------
        if let Some((r, value)) = self.pending_register_write {
            trace!("Setting register: {:?} = {:>02X}", r, value);
            self.pending_register_write = None;
            self.reg.set(r, value);
        }
        if let Some((co, zo, no)) = self.pending_flag_write {
            self.reg.update_co(co);
            self.reg.update_zo(zo);
            self.reg.update_no(no);
        }
        // ------------------------------------------------------------
        // Use microprogram word from last iteration
        // ------------------------------------------------------------
        let mp_ram_out = self.mp_ram.get().clone();
        let mut sig = Signal::new(&mp_ram_out, &self.current_instruction);
        trace!("Address: {:>08b} ({0})", self.mp_ram.current_addr());
        trace!("Word: {:?}", mp_ram_out);
        // ------------------------------------------------------------
        // Add inputs to sig
        // ------------------------------------------------------------
        sig.set_edge_int(self.input_edge_int);
        sig.set_level_int(self.input_level_int);
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
                0b0000_0000
                    + ((sig.mrgab2() as u8) << 2)
                    + ((sig.mrgab1() as u8) << 1)
                    + sig.mrgab0() as u8
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
        }
        // Clearing edge interrupt if used
        if self.input_edge_int && sig.na0() && sig.mac0() && sig.mac1() {
            trace!("Clearing edge interrupt");
            self.input_edge_int = false;
        }
        // ------------------------------------------------------------
        // Select next microprogram word
        // ------------------------------------------------------------
        let next_mp_ram_addr = MicroprogramRam::get_addr(&sig);
        trace!("Next MP_RAM address: {}", next_mp_ram_addr);
        self.mp_ram.select(next_mp_ram_addr);
    }
}

impl Widget for Machine {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        let in_fc = self.bus.read(0xFC).display();
        let in_fd = self.bus.read(0xFD).display();
        let in_fe = self.bus.read(0xFE).display();
        let in_ff = self.bus.read(0xFF).display();
        let out_fe = self.bus.output_fe().display();
        let out_ff = self.bus.output_ff().display();

        let x = area.x + 1;
        let y = area.y + 1;

        let dimmed = Style::default().modifier(Modifier::DIM);
        //
        // Frequency
        buf.set_string(
            area.width - 20,
            1,
            format!("Frequency: {:>7}KHz", self.measured_frequency as u64),
            dimmed,
        );

        // Output register
        buf.set_string(x, y, "Outputs:", dimmed);
        display_u8_str(buf, x, y + 1, out_ff);
        display_u8_str(buf, x + 9, y + 1, out_fe);
        buf.set_string(x + 6, y + 2, "FF", dimmed);
        buf.set_string(x + 15, y + 2, "FE", dimmed);

        // Input register
        buf.set_string(x, y + 4, "Inputs:", dimmed);
        display_u8_str(buf, x, y + 5, in_ff);
        display_u8_str(buf, x + 9, y + 5, in_fe);
        display_u8_str(buf, x + 18, y + 5, in_fd);
        display_u8_str(buf, x + 27, y + 5, in_fc);
        buf.set_string(x + 6, y + 6, "FF", dimmed);
        buf.set_string(x + 15, y + 6, "FE", dimmed);
        buf.set_string(x + 24, y + 6, "FD", dimmed);
        buf.set_string(x + 33, y + 6, "FC", dimmed);
    }
}

/// Display `1`s in yellow and `0`s in gray.
fn display_u8_str(buf: &mut Buffer, x: u16, y: u16, s: String) {
    let mut v = 0;
    s.chars().for_each(|c| {
        let style = match c {
            '1' | '●' => Style::default().fg(Color::Yellow),
            '0' | '○' => Style::default().fg(Color::Gray),
            _ => Style::default(),
        };
        buf.set_string(x + v, y, c.to_string(), style);
        v += 1;
    });
}
