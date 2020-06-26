use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
use tui::widgets::Widget;

use std::f32::consts::FRAC_PI_2;
use std::ops::{Deref, DerefMut};

use crate::helpers::{self, Configuration};
use crate::machine::DASR;
use crate::supervisor::Supervisor;
use crate::tui::display::Display;

const MINIMUM_ALLOWED_WIDTH_FOR_MEMORY_DISPLAY: u16 = 51;
const MINIMUM_ALLOWED_HEIGHT_FOR_MEMORY_DISPLAY: u16 = 27;

pub struct SupervisorWrapper {
    supervisor: Supervisor,
    /// The part currently displayed by the TUI.
    part: Part,
    /// Counting draw cycles.
    draw_counter: usize,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Part {
    RegisterBlock,
    Memory,
}

impl SupervisorWrapper {
    pub fn new(conf: &Configuration) -> Self {
        SupervisorWrapper {
            part: Part::RegisterBlock,
            supervisor: Supervisor::new(conf),
            draw_counter: 0,
        }
    }
    pub fn show(&mut self, part: Part) {
        self.part = part;
    }
    /// Show the memory.
    fn show_part_memory(&mut self, area: Rect, buf: &mut Buffer, x: u16, y: u16) {
        buf.set_string(x, y + 8, "Memory:", *helpers::DIMMED);
        if area.width < MINIMUM_ALLOWED_WIDTH_FOR_MEMORY_DISPLAY {
            buf.set_string(x, y + 10, "Display width too small!", *helpers::LIGHTRED);
        } else if area.height < MINIMUM_ALLOWED_HEIGHT_FOR_MEMORY_DISPLAY {
            buf.set_string(x, y + 10, "Display height too small!", *helpers::LIGHTRED);
        } else {
            for i in 0..=0xFB {
                let data = self.machine().bus.read(i);
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
    }
    /// Show the register block.
    fn show_part_register_block(&mut self, _area: Rect, buf: &mut Buffer, x: u16, y: u16) {
        // Register block
        buf.set_string(x, y + 8, "Registers:", *helpers::DIMMED);
        for (index, content) in self.machine().registers().content.iter().enumerate() {
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
}

impl Widget for SupervisorWrapper {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        let in_fc = self.machine().bus.read(0xFC).display();
        let in_fd = self.machine().bus.read(0xFD).display();
        let in_fe = self.machine().bus.read(0xFE).display();
        let in_ff = self.machine().bus.read(0xFF).display();
        let out_fe = self.machine().bus.output_fe().display();
        let out_ff = self.machine().bus.output_ff().display();
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

        match self.part {
            Part::Memory => self.show_part_memory(area, buf, x, y),
            Part::RegisterBlock => self.show_part_register_block(area, buf, x, y),
        }

        let board = self.machine().bus().board();

        // Details
        if area.width >= 46 && area.height >= 19 {
            if *board.fan_rpm() != 0 {
                if self.draw_counter % 10 <= 5 {
                    let s = format!("{:>4} RPM ×", board.fan_rpm());
                    buf.set_string(area.width - 9, area.y, s, *helpers::YELLOW);
                } else {
                    let s = format!("{:>4} RPM +", board.fan_rpm());
                    buf.set_string(area.width - 9, area.y, s, *helpers::YELLOW);
                }
            }
            if *board.irg() != 0 {
                let s = format!("{:>02X}  IRG", board.irg());
                buf.set_string(area.width - 7, area.y + 3, "0x", *helpers::DIMMED);
                buf.set_string(area.width - 5, area.y + 3, s, *helpers::YELLOW);
            }
            if *board.org1() != 0 {
                let s = format!("{:>02X} ORG1", board.org1());
                buf.set_string(area.width - 7, area.y + 4, "0x", *helpers::DIMMED);
                buf.set_string(area.width - 5, area.y + 4, s, *helpers::YELLOW);
            }
            if *board.org2() != 0 {
                let s = format!("{:>02X} ORG2", board.org2());
                buf.set_string(area.width - 7, area.y + 5, "0x", *helpers::DIMMED);
                buf.set_string(area.width - 5, area.y + 5, s, *helpers::YELLOW);
            }
            if (*board.temp() - FRAC_PI_2).abs() > 0.01 {
                buf.set_string(area.width - 2, area.y + 7, "TEMP", *helpers::YELLOW);
            }

            if board.analog_inputs()[0] != 0.0 {
                let s = format!("{:.1}V AI1", board.analog_inputs()[0]);
                buf.set_string(area.width - 6, area.y + 9, s, *helpers::YELLOW);
            }
            if board.analog_inputs()[1] != 0.0 {
                let s = format!("{:.1}V AI2", board.analog_inputs()[1]);
                buf.set_string(area.width - 6, area.y + 10, s, *helpers::YELLOW);
            }
            if board.analog_outputs()[0] != 0.0 {
                let s = format!("{:.1}V AO1", board.analog_outputs()[0]);
                buf.set_string(area.width - 6, area.y + 11, s, *helpers::YELLOW);
            }
            if board.analog_outputs()[1] != 0.0 {
                let s = format!("{:.1}V AO2", board.analog_outputs()[1]);
                buf.set_string(area.width - 6, area.y + 12, s, *helpers::YELLOW);
            }
            let uio1 = board.dasr().contains(DASR::UIO_1);
            let uio2 = board.dasr().contains(DASR::UIO_2);
            let uio3 = board.dasr().contains(DASR::UIO_3);
            if board.uio_dir()[0] && uio1 {
                let s = format!("« {} UIO1", uio1 as u8);
                buf.set_string(area.width - 6, area.y + 13, s, *helpers::YELLOW);
            } else if uio1 {
                let s = format!("» {} UIO1", uio1 as u8);
                buf.set_string(area.width - 6, area.y + 13, s, *helpers::YELLOW);
            }
            if board.uio_dir()[1] && uio2 {
                let s = format!("« {} UIO2", uio2 as u8);
                buf.set_string(area.width - 6, area.y + 14, s, *helpers::YELLOW);
            } else if uio2 {
                let s = format!("» {} UIO2", uio2 as u8);
                buf.set_string(area.width - 6, area.y + 14, s, *helpers::YELLOW);
            }
            if board.uio_dir()[2] && uio3 {
                let s = format!("« {} UIO3", uio3 as u8);
                buf.set_string(area.width - 6, area.y + 15, s, *helpers::YELLOW);
            } else if uio3 {
                let s = format!("» {} UIO3", uio3 as u8);
                buf.set_string(area.width - 6, area.y + 15, s, *helpers::YELLOW);
            }

            if board.dasr().contains(DASR::J1) {
                buf.set_string(area.width - 4, area.height, "╼━╾ J1", *helpers::GREEN);
            }
            if !board.dasr().contains(DASR::J2) {
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

impl Deref for SupervisorWrapper {
    type Target = Supervisor;
    fn deref(&self) -> &Self::Target {
        &self.supervisor
    }
}

impl DerefMut for SupervisorWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.supervisor
    }
}
