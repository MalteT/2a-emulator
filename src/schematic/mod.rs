use mr2a_asm_parser::asm::{Asm, Register as RegNum};
use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
use tui::widgets::Widget;

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
    pending_register_write: Option<(RegNum, u8)>,
    input_edge_int: bool,
    input_level_int: bool,
}

impl Machine {
    /// Create a new Minirechner 2a
    pub fn new() -> Self {
        let mp_ram = MicroprogramRam::new();
        let reg = Register::new();
        let bus = Bus::new();
        let current_instruction = Instruction::reset();
        let pending_register_write = None;
        let input_edge_int = false;
        let input_level_int = false;
        Machine {
            mp_ram,
            reg,
            bus,
            current_instruction,
            pending_register_write,
            input_edge_int,
            input_level_int,
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
    pub fn clk(&mut self, _sig: bool) {
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
    pub fn reset(&mut self, _sig: bool) {
        self.reg.reset();
        self.current_instruction = Instruction::reset();
        self.mp_ram.reset();
    }
    /// Input an edge interrupt.
    pub fn edge_int(&mut self) {
        eprintln!("MACHINE: Received edge interrupt");
        self.input_edge_int = true;
    }
    /// TODO: Dummy
    pub fn show(&mut self, _part: Part) {}
    /// Update the machine.
    /// This should be equivalent to a CLK signal on the real machine.
    fn update(&mut self) {
        // ------------------------------------------------------------
        // Update register block with value from last iteration
        // ------------------------------------------------------------
        if let Some((r, value)) = self.pending_register_write {
            self.pending_register_write = None;
            self.reg.set(r, value);
        }
        // ------------------------------------------------------------
        // Use microprogram word from last iteration
        // ------------------------------------------------------------
        let mp_ram_out = self.mp_ram.get().clone();
        let mut sig = Signal::new(&mp_ram_out, &self.current_instruction);
        eprintln!("UPDATE: Instruction: {:?}", self.current_instruction);
        eprintln!("UPDATE: Word: {:?}", mp_ram_out);
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
        eprintln!(
            "UPDATE: Output Registers: A: 0x{:>02X}, B: 0x{:>02X}",
            doa, dob
        );
        // ------------------------------------------------------------
        // Read value from bus at selected address
        // ------------------------------------------------------------
        let mut bus_content = 0;
        if sig.busen() {
            bus_content = self.bus.read(doa);
            eprintln!(
                "UPDATE: Read 0x{:>02X} from bus address 0x{:>02X}",
                bus_content, doa
            );
        }
        // ------------------------------------------------------------
        // Update current instruction from bus
        // ------------------------------------------------------------
        if sig.mac1() && sig.mac2() {
            // Reset the instruction register
            eprintln!("UPDATE: Resetting instruction register");
            self.current_instruction = Instruction::reset();
            sig.set_current_instruction(&self.current_instruction);
        } else if sig.mac0() && sig.mac2() {
            // Selecting next instruction
            eprintln!("UPDATE: Selecting next instruction");
            self.current_instruction = Instruction::from_bits_truncate(bus_content);
            sig.set_current_instruction(&self.current_instruction);
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
        eprintln!("UPDATE: ALU function: {:?}", alu_fn);
        let memdo = Alu::output(sig.cf(), alu_in_a, alu_in_b, alu_fn);
        let co = Alu::co(sig.cf(), alu_in_a, alu_in_b, alu_fn);
        let zo = Alu::zo(sig.cf(), alu_in_a, alu_in_b, alu_fn);
        let no = Alu::no(sig.cf(), alu_in_a, alu_in_b, alu_fn);
        eprintln!("UPDATE: ALU calculated: 0x{:>02X}", memdo);
        // ------------------------------------------------------------
        // Add ALU outputs to signals
        // ------------------------------------------------------------
        sig.set_co(co);
        sig.set_zo(zo);
        sig.set_no(no);
        // Update registers if necessary
        if sig.mrgwe() {
            eprintln!("UPDATE: Changing registers");
            self.reg.write(&sig, memdo);
        }
        if sig.mchflg() {
            eprintln!("UPDATE: Updating flags");
            self.reg.write_flags(&sig);
        }
        // ------------------------------------------------------------
        // Update bus
        // TODO: wait
        // ------------------------------------------------------------
        if sig.buswr() {
            eprintln!(
                "UPDATE: Writing 0x{:>02X} to bus address 0x{:>02X}",
                memdo, doa
            );
            self.bus.write(doa, memdo);
        }
        // Clearing edge interrupt if used
        if self.input_edge_int && sig.na0() && sig.mac0() && sig.mac1() {
            eprintln!("UPDATE: Clearing edge interrupt");
            self.input_edge_int = false;
        }
        // ------------------------------------------------------------
        // Select next microprogram word
        // ------------------------------------------------------------
        eprintln!("UPDATE: Selecting next mp_ram word");
        let next_mp_ram_addr = self.mp_ram.next_addr(&sig);
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
        // let mut x = area.x;
        // let mut y = area.y;
        // match self.displaying_part {
        //     Part::InterruptLogic => {
        //         let il1 = self.get_part(Part::Il1).borrow().to_utf8_string();
        //         let il2 = self.get_part(Part::Il2).borrow().to_utf8_string();
        //         let iff1 = self.get_part(Part::Iff1).borrow().to_utf8_string();
        //         let iff2 = self.get_part(Part::Iff2).borrow().to_utf8_string();
        //         let mut s: StrGrid = include_str!("../../displays/interrupt.utf8.template")
        //             .try_into()
        //             .unwrap();
        //         s.put(1, &il1).expect("il1 fits into interruptlogic");
        //         s.put(2, &iff2).expect("iff2 fits into interruptlogic");
        //         s.put(3, &il2).expect("il2 fits into interruptlogic");
        //         s.put(4, &iff1).expect("iff1 fits into interruptlogic");
        //         s.to_string()
        //     }
        //     _ => self
        //         .get_part(self.displaying_part)
        //         .borrow()
        //         .to_utf8_string(),
        // }
        // .lines()
        // .take(area.height as usize)
        // .for_each(|line| {
        //     x = area.x;
        //     line.char_indices()
        //         .take(area.width as usize)
        //         .for_each(|(_, c)| {
        //             let style = match c {
        //                 '○' => Style::default().fg(Color::Gray),
        //                 '●' => Style::default().fg(Color::Yellow),
        //                 _ => Style::default(),
        //             };
        //             buf.set_string(x, y, c.to_string(), style);
        //             x += 1;
        //         });
        //     y += 1;
        // });
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
