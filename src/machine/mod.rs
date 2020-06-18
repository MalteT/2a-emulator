//! The actual emulated machine.

use log::trace;
use parser2a::asm::{Asm, Stacksize};
use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
use tui::widgets::Widget;

use std::f32::consts::FRAC_PI_2;

mod alu;
mod board;
mod bus;
mod instruction;
mod mp_ram;
mod register;
mod signal;

pub use alu::Alu;
pub use bus::Bus;
pub use instruction::Instruction;
pub use mp_ram::{MP28BitWord, MicroprogramRam};
pub use register::{Register, RegisterNumber};
pub use signal::Signal;

use crate::compiler::Translator;
use crate::helpers;
use crate::helpers::Configuration;
use crate::machine::board::DASR;
use crate::tui::display::Display;

const MINIMUM_ALLOWED_WIDTH_FOR_MEMORY_DISPLAY: u16 = 51;
const MINIMUM_ALLOWED_HEIGHT_FOR_MEMORY_DISPLAY: u16 = 27;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Part {
    RegisterBlock,
    Memory,
}

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
    /// The part to show in the UI
    showing: Part,
}

impl Machine {
    /// Create and run a new Minirechner 2a with an optional program.
    pub fn new(program: Option<&Asm>, conf: &Configuration) -> Self {
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
        let showing = Part::RegisterBlock;
        let state = State::Running;
        let mut machine = Machine {
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
            showing,
        };
        // Load program if given any
        if let Some(program) = program {
            let bytecode = Translator::compile(program);
            // Safe the lines for later
            for (line, bytes) in &bytecode.lines {
                machine.program_lines.push((line.to_string(), bytes.len()));
            }
            for (address, byte) in bytecode.bytes().enumerate() {
                machine.bus.write(address as u8, *byte);
            }
            machine.stacksize = Some(bytecode.stacksize);
        }
        // Apply configuration
        machine.bus.board.set_irg(conf.irg);
        machine.bus.board.set_temp(conf.temp);
        machine.bus.board.set_j1(conf.jumper[0]);
        machine.bus.board.set_j2(conf.jumper[1]);
        machine.bus.board.set_i1(conf.analog_inputs[0]);
        machine.bus.board.set_i2(conf.analog_inputs[1]);
        if let Some(val) = conf.uios[0] {
            machine.bus.board.set_uio1(val);
        }
        if let Some(val) = conf.uios[1] {
            machine.bus.board.set_uio2(val);
        }
        if let Some(val) = conf.uios[2] {
            machine.bus.board.set_uio3(val);
        }
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
    /// Get the currently executed lines of the program.
    ///
    /// # Arguments
    /// - `context` The amount of lines before and after the currently executed line.
    ///
    /// # Returns
    /// - A tuple with a list of [`String`]s of asm lines and the index of the one
    /// currently executed by the machine.
    pub fn get_current_lines(&self, context: isize) -> (usize, Vec<&String>) {
        // If no program is loaded, no lines are available, prevent errors
        if self.program_lines.is_empty() {
            return (0, vec![]);
        }
        let current_byte_index = self.reg.get(RegisterNumber::R3) as isize;
        // Find current line
        let mut counter = current_byte_index;
        let mut index: isize = 0;
        while counter >= 0 && index < self.program_lines.len() as isize {
            counter -= self.program_lines[index as usize].1 as isize;
            if counter >= 0 {
                index += 1;
            }
        }
        let mut middle = context;
        // Find left border
        let left = if index - context >= 0 {
            (index - context) as usize
        } else {
            middle += index - context;
            0
        };
        // Find right border
        let right = if index + context < self.program_lines.len() as isize {
            (index + context) as usize
        } else {
            self.program_lines.len() - 1
        };
        let ret: Vec<_> = self.program_lines.iter().map(|(x, _)| x).collect();
        (middle as usize, ret[left..=right].into())
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
    /// Select the element to show in the TUI.
    pub fn show(&mut self, part: Part) {
        self.showing = part;
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

impl Widget for Machine {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        let in_fc = self.bus.read(0xFC).display();
        let in_fd = self.bus.read(0xFD).display();
        let in_fe = self.bus.read(0xFE).display();
        let in_ff = self.bus.read(0xFF).display();
        let out_fe = self.bus.output_fe().display();
        let out_ff = self.bus.output_ff().display();
        self.draw_counter = self.draw_counter.overflowing_add(1).0;

        let x = area.x + 1;
        let y = area.y + 1;

        let dimmed = Style::default().modifier(Modifier::DIM);

        // Output register
        buf.set_string(x, y, "Outputs:", dimmed);
        display_u8_str(buf, x, y + 1, out_ff);
        display_u8_str(buf, x + 9, y + 1, out_fe);
        buf.set_string(x + 6, y + 2, "FF", Style::default());
        buf.set_string(x + 15, y + 2, "FE", Style::default());

        // Input register
        buf.set_string(x, y + 4, "Inputs:", dimmed);
        display_u8_str(buf, x, y + 5, in_ff);
        display_u8_str(buf, x + 9, y + 5, in_fe);
        display_u8_str(buf, x + 18, y + 5, in_fd);
        display_u8_str(buf, x + 27, y + 5, in_fc);
        buf.set_string(x + 6, y + 6, "FF", Style::default());
        buf.set_string(x + 15, y + 6, "FE", Style::default());
        buf.set_string(x + 24, y + 6, "FD", Style::default());
        buf.set_string(x + 33, y + 6, "FC", Style::default());

        if self.showing == Part::Memory {
            buf.set_string(x, y + 8, "Memory:", dimmed);
            if area.width < MINIMUM_ALLOWED_WIDTH_FOR_MEMORY_DISPLAY {
                buf.set_string(x, y + 10, "Display width too small!", *helpers::LIGHTRED);
            } else if area.height < MINIMUM_ALLOWED_HEIGHT_FOR_MEMORY_DISPLAY {
                buf.set_string(x, y + 10, "Display height too small!", *helpers::LIGHTRED);
            } else {
                for i in 0..=0xFB {
                    let data = self.bus.read(i);
                    let color = if data == 0 {
                        Default::default()
                    } else {
                        *helpers::YELLOW
                    };
                    let data = format!("{:>02X}", data);
                    let x_pos = x + 2 + (i as u16 % 0x10) * 3;
                    let y_pos = y + 10 + i as u16 / 0x10;
                    let width = if area.width > x { area.width - x } else { 0 };
                    buf.set_stringn(x_pos, y_pos, &data, width as usize, color);
                    if i <= 0xF {
                        let nr = format!("{:>2X}", i);
                        buf.set_stringn(x_pos, y_pos - 1, &nr, width as usize, *helpers::DIMMED);
                    }
                    if i % 0x10 == 0 {
                        let nr = format!("{:>2X}", i / 0x10);
                        buf.set_stringn(x_pos - 3, y_pos, &nr, width as usize, *helpers::DIMMED);
                    }
                }
            }
        } else if self.showing == Part::RegisterBlock {
            // Register block
            buf.set_string(x, y + 8, "Registers:", dimmed);
            for (index, content) in self.reg.content.iter().enumerate() {
                let reg = match index {
                    3 => "PC".to_owned(),
                    4 => "FR".to_owned(),
                    5 => "SP".to_owned(),
                    i => format!("R{}", i),
                };
                if index <= 3 {
                    buf.set_string(x, y + 9 + index as u16, reg, Style::default());
                    display_u8_str(buf, x + 3, y + 9 + index as u16, content.display());
                } else {
                    buf.set_string(x, y + 9 + index as u16, reg, *helpers::DIMMED);
                    buf.set_string(
                        x + 3,
                        y + 9 + index as u16,
                        content.display(),
                        *helpers::DIMMED,
                    );
                };
            }
        }

        // Details
        if area.width >= 46 && area.height >= 19 {
            if self.bus.board.fan_rpm != 0 {
                if self.draw_counter % 10 <= 5 {
                    let s = format!("{:>4} RPM ×", self.bus.board.fan_rpm);
                    buf.set_string(area.width - 9, area.y, s, *helpers::YELLOW);
                } else {
                    let s = format!("{:>4} RPM +", self.bus.board.fan_rpm);
                    buf.set_string(area.width - 9, area.y, s, *helpers::YELLOW);
                }
            }
            if self.bus.board.irg != 0 {
                let s = format!("{:>02X}  IRG", self.bus.board.irg);
                buf.set_string(area.width - 7, area.y + 3, "0x", *helpers::DIMMED);
                buf.set_string(area.width - 5, area.y + 3, s, *helpers::YELLOW);
            }
            if self.bus.board.org1 != 0 {
                let s = format!("{:>02X} ORG1", self.bus.board.org1);
                buf.set_string(area.width - 7, area.y + 4, "0x", *helpers::DIMMED);
                buf.set_string(area.width - 5, area.y + 4, s, *helpers::YELLOW);
            }
            if self.bus.board.org2 != 0 {
                let s = format!("{:>02X} ORG2", self.bus.board.org2);
                buf.set_string(area.width - 7, area.y + 5, "0x", *helpers::DIMMED);
                buf.set_string(area.width - 5, area.y + 5, s, *helpers::YELLOW);
            }
            if (self.bus.board.temp - FRAC_PI_2).abs() > 0.01 {
                buf.set_string(area.width - 2, area.y + 7, "TEMP", *helpers::YELLOW);
            }

            if self.bus.board.analog_inputs[0] != 0.0 {
                let s = format!("{:.1}V AI1", self.bus.board.analog_inputs[0]);
                buf.set_string(area.width - 6, area.y + 9, s, *helpers::YELLOW);
            }
            if self.bus.board.analog_inputs[1] != 0.0 {
                let s = format!("{:.1}V AI2", self.bus.board.analog_inputs[1]);
                buf.set_string(area.width - 6, area.y + 10, s, *helpers::YELLOW);
            }
            if self.bus.board.analog_outputs[0] != 0.0 {
                let s = format!("{:.1}V AO1", self.bus.board.analog_outputs[0]);
                buf.set_string(area.width - 6, area.y + 11, s, *helpers::YELLOW);
            }
            if self.bus.board.analog_outputs[1] != 0.0 {
                let s = format!("{:.1}V AO2", self.bus.board.analog_outputs[1]);
                buf.set_string(area.width - 6, area.y + 12, s, *helpers::YELLOW);
            }
            let uio1 = self.bus.board.dasr.contains(DASR::UIO_1);
            let uio2 = self.bus.board.dasr.contains(DASR::UIO_2);
            let uio3 = self.bus.board.dasr.contains(DASR::UIO_3);
            if self.bus.board.uio_dir[0] && uio1 {
                let s = format!("« {} UIO1", uio1 as u8);
                buf.set_string(area.width - 6, area.y + 13, s, *helpers::YELLOW);
            } else if uio1 {
                let s = format!("» {} UIO1", uio1 as u8);
                buf.set_string(area.width - 6, area.y + 13, s, *helpers::YELLOW);
            }
            if self.bus.board.uio_dir[1] && uio2 {
                let s = format!("« {} UIO2", uio2 as u8);
                buf.set_string(area.width - 6, area.y + 14, s, *helpers::YELLOW);
            } else if uio2 {
                let s = format!("» {} UIO2", uio2 as u8);
                buf.set_string(area.width - 6, area.y + 14, s, *helpers::YELLOW);
            }
            if self.bus.board.uio_dir[2] && uio3 {
                let s = format!("« {} UIO3", uio3 as u8);
                buf.set_string(area.width - 6, area.y + 15, s, *helpers::YELLOW);
            } else if uio3 {
                let s = format!("» {} UIO3", uio3 as u8);
                buf.set_string(area.width - 6, area.y + 15, s, *helpers::YELLOW);
            }

            if self.bus.board.dasr.contains(DASR::J1) {
                buf.set_string(area.width - 4, area.height, "╼━╾ J1", *helpers::GREEN);
            }
            if !self.bus.board.dasr.contains(DASR::J2) {
                buf.set_string(
                    area.width - 4,
                    area.height + 1,
                    "╼ ╾ J2",
                    *helpers::LIGHTRED,
                );
            }
        } else {
            buf.set_string(area.width - 1, area.height + 1, "...", *helpers::DIMMED);
        }
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
