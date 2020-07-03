use tui::buffer::Buffer;
use tui::layout::{Margin, Rect};
use tui::style::{Color, Style};
use tui::widgets::StatefulWidget;

use std::ops::{Deref, DerefMut};

use crate::{
    args::InitialMachineConfiguration,
    helpers,
    supervisor::Supervisor,
    tui::{display::Display, BoardInfoSidebarWidget},
};

const MINIMUM_ALLOWED_WIDTH_FOR_MEMORY_DISPLAY: u16 = 51;
const MINIMUM_ALLOWED_HEIGHT_FOR_MEMORY_DISPLAY: u16 = 27;
const ONE_SPACE: u16 = 1;
const BYTE_WIDTH: u16 = 8;
const OUTPUT_REGISTER_WIDGET_WIDTH: u16 = 2 * BYTE_WIDTH + ONE_SPACE;
const OUTPUT_REGISTER_WIDGET_HEIGHT: u16 = 3;
const INPUT_REGISTER_WIDGET_WIDTH: u16 = 4 * BYTE_WIDTH + 3 * ONE_SPACE;
const INPUT_REGISTER_WIDGET_HEIGHT: u16 = 3;
const BOARD_INFO_SIDEBAR_WIDGET_WIDTH: u16 = 20;

pub struct SupervisorWrapper;

impl SupervisorWrapper {
    pub fn new() -> Self {
        SupervisorWrapper
    }
}

pub struct SupervisorWrapperState {
    pub supervisor: Supervisor,
    /// The part currently displayed by the TUI.
    pub part: Part,
    /// Counting draw cycles.
    pub draw_counter: usize,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Part {
    RegisterBlock,
    Memory,
}

impl SupervisorWrapperState {
    pub fn new(conf: &InitialMachineConfiguration) -> Self {
        SupervisorWrapperState {
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

impl SupervisorWrapper {
    fn render_output_registers(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut SupervisorWrapperState,
    ) {
        // Fetch output register values
        let out_fe = state.machine().bus.output_fe();
        let out_ff = state.machine().bus.output_ff();
        // Calculate area
        let inner_area = Rect {
            width: OUTPUT_REGISTER_WIDGET_WIDTH,
            height: OUTPUT_REGISTER_WIDGET_HEIGHT,
            ..area
        };
        // Draw!
        OutputRegisterWidget.render(inner_area, buf, &mut (out_fe, out_ff));
    }
    fn render_input_registers(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut SupervisorWrapperState,
    ) {
        // Fetch input register values
        let in_fc = state.machine().bus.read(0xFC);
        let in_fd = state.machine().bus.read(0xFD);
        let in_fe = state.machine().bus.read(0xFE);
        let in_ff = state.machine().bus.read(0xFF);
        // Calculate area
        let inner_area = Rect {
            y: area.y + OUTPUT_REGISTER_WIDGET_HEIGHT + ONE_SPACE,
            width: INPUT_REGISTER_WIDGET_WIDTH,
            height: INPUT_REGISTER_WIDGET_HEIGHT,
            ..area
        };
        // Draw!
        InputRegisterWidget.render(inner_area, buf, &mut (in_fc, in_fd, in_fe, in_ff));
    }
    fn render_board_info_sidebar(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut SupervisorWrapperState,
    ) {
        if area.width > INPUT_REGISTER_WIDGET_WIDTH + BOARD_INFO_SIDEBAR_WIDGET_WIDTH {
            // Actually draw the information
            let sidebar_area = Rect {
                x: area.x + area.width - BOARD_INFO_SIDEBAR_WIDGET_WIDTH,
                width: BOARD_INFO_SIDEBAR_WIDGET_WIDTH,
                ..area
            };
            BoardInfoSidebarWidget.render(sidebar_area, buf, state)
        } else {
            // There's not enough space. Show a hint, that not everything is displayed.
            buf.set_string(area.right() - 3, area.bottom() - 1, "...", *helpers::DIMMED);
        }
    }
}

/// Draw the input register content.
///
/// # Example
/// ```
/// Inputs:
/// 00000000 00000000 00010100 00001010
///       FF       FE       FD       FC
/// ```
struct InputRegisterWidget;

impl StatefulWidget for InputRegisterWidget {
    /// Input registers FC, FD, FE, FF.
    type State = (u8, u8, u8, u8);

    fn render(self, area: Rect, buf: &mut Buffer, (fc, fd, fe, ff): &mut Self::State) {
        // Some helper constants
        const LABEL_OFFSET: u16 = 6;
        const BYTE_SPACE: u16 = BYTE_WIDTH + ONE_SPACE;
        // Make sure everything is fine. This should never fail, as
        // the interface does not draw unless a certain size is available.
        debug_assert!(area.width >= INPUT_REGISTER_WIDGET_WIDTH);
        debug_assert!(area.height >= INPUT_REGISTER_WIDGET_HEIGHT);
        // Display the "Inputs" header
        buf.set_string(area.x, area.y, "Inputs:", *helpers::DIMMED);
        // Display all the registers in binary
        render_byte(buf, area.x, area.y + 1, *ff);
        render_byte(buf, area.x + BYTE_SPACE, area.y + 1, *fe);
        render_byte(buf, area.x + 2 * BYTE_SPACE, area.y + 1, *fd);
        render_byte(buf, area.x + 3 * BYTE_SPACE, area.y + 1, *fc);
        buf.set_string(area.x + LABEL_OFFSET, area.y + 2, "FF", *helpers::DIMMED);
        buf.set_string(
            area.x + LABEL_OFFSET + BYTE_SPACE,
            area.y + 2,
            "FE",
            *helpers::DIMMED,
        );
        buf.set_string(
            area.x + LABEL_OFFSET + 2 * BYTE_SPACE,
            area.y + 2,
            "FD",
            *helpers::DIMMED,
        );
        buf.set_string(
            area.x + LABEL_OFFSET + 3 * BYTE_SPACE,
            area.y + 2,
            "FC",
            *helpers::DIMMED,
        );
    }
}

/// Draw the output register content.
///
/// # Example
/// ```
/// Outputs:
/// 00011110 00000000
///       FF       FE
/// ```
struct OutputRegisterWidget;

impl StatefulWidget for OutputRegisterWidget {
    /// Output registers FE and FF
    type State = (u8, u8);

    fn render(self, area: Rect, buf: &mut Buffer, (fe, ff): &mut Self::State) {
        // Make sure everything is fine. This should never fail, as
        // the interface does not draw unless a certain size is available.
        debug_assert!(area.width >= OUTPUT_REGISTER_WIDGET_WIDTH);
        debug_assert!(area.height >= OUTPUT_REGISTER_WIDGET_HEIGHT);
        buf.set_string(area.x, area.x, "Outputs:", *helpers::DIMMED);
        render_byte(buf, area.x, area.y + 1, *ff);
        render_byte(buf, area.x + 9, area.y + 1, *fe);
        buf.set_string(area.x + 6, area.y + 2, "FF", *helpers::DIMMED);
        buf.set_string(area.x + 15, area.y + 2, "FE", *helpers::DIMMED);
    }
}

impl StatefulWidget for SupervisorWrapper {
    type State = SupervisorWrapperState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Leave some space between the border and everything else
        let area = area.inner(&Margin {
            vertical: 1,
            horizontal: 1,
        });
        // Render all the different parts interface.
        self.render_output_registers(area, buf, state);
        self.render_input_registers(area, buf, state);
        self.render_board_info_sidebar(area, buf, state);

        match state.part {
            Part::Memory => state.show_part_memory(area, buf, area.x, area.y),
            Part::RegisterBlock => state.show_part_register_block(area, buf, area.x, area.y),
        }

        // Update draw_counter
        state.draw_counter = state.draw_counter.overflowing_add(1).0;
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

fn render_byte(buf: &mut Buffer, x: u16, y: u16, byte: u8) {
    let style = if byte == 0 {
        Style::default()
    } else {
        *helpers::BOLD
    };
    let string = format!("{:>08b}", byte);
    buf.set_string(x, y, string, style)
}

impl Deref for SupervisorWrapperState {
    type Target = Supervisor;
    fn deref(&self) -> &Self::Target {
        &self.supervisor
    }
}

impl DerefMut for SupervisorWrapperState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.supervisor
    }
}
