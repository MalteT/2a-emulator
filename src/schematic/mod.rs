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
    reg: Register,
    bus: Bus,
}

impl Machine {
    /// Create a new Minirechner 2a
    pub fn new() -> Self {
        let mp_ram = MicroprogramRam::new();
        let reg = Register::new();
        let bus = Bus::new();
        Machine { mp_ram, reg, bus }
    }
    /// TODO: Dummy
    pub fn clk(&mut self, _sig: bool) {
        let x = self.bus.output_ff().overflowing_add(1);
        self.bus.write(0xff, x.0);
    }
    /// TODO: Dummy
    pub fn reset(&mut self, _sig: bool) {}
    /// TODO: Dummy
    pub fn edge_int(&mut self, _sig: bool) {}
    /// TODO: Dummy
    pub fn show(&mut self, _part: Part) {}
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
