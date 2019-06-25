use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::widgets::Widget;

use std::cell::RefCell;
use std::convert::TryInto;
use std::rc::Rc;

mod fns;
mod mp_ram;

use crate::tui::grid::StrGrid;
pub use fns::*;

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
pub struct Machine { }

pub enum Signal {
    Rising,
    Falling,
    High,
    Low,
}

impl Machine {
    /// Create a new Minirechner 2a
    pub fn new() -> Self {
        Machine {}
    }
    /// TODO: Dummy
    pub fn clk(&mut self, sig: bool) {
        println!("Implement Machine::clk");
    }
    /// TODO: Dummy
    pub fn reset(&mut self, sig: bool) {
        println!("Implement Machine::reset");
    }
    /// TODO: Dummy
    pub fn edge_int(&mut self, sig: bool) {
        println!("Implement Machine::edge_int");
    }
    /// TODO: Dummy
    pub fn show(&mut self, part: Part) {
        println!("Implement Machine::show");
    }
}

impl Widget for Machine {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        buf.set_string(area.x, area.y, "Nothing to see in this machine", Style::default());
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
