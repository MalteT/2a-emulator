//! Everything related to drawing the [`RegisterBlockWidget`].
use tui::{buffer::Buffer, layout::Rect, style::Style, widgets::Widget};

use crate::{helpers, machine::Register, tui::display::Display};

/// A widget for displaying the RegisterBlock.
pub struct RegisterBlockWidget<'a>(pub &'a Register);

impl<'a> Widget for RegisterBlockWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Register block
        buf.set_string(area.left(), area.top(), "Registers:", *helpers::DIMMED);
        for (index, content) in self.0.content().iter().enumerate() {
            // Give some registers special names
            let reg = match index {
                3 => "PC".to_owned(),
                4 => "FR".to_owned(),
                5 => "SP".to_owned(),
                i => format!("R{}", i),
            };
            // Dimm the register if the user cannot influence it's value and
            // make it bold if the former does not apply and the content is not zero.
            let style = if index > 3 {
                *helpers::DIMMED
            } else if *content == 0 {
                Style::default()
            } else {
                *helpers::BOLD
            };
            // Display register name
            buf.set_string(area.left(), area.top() + 1 + index as u16, reg, style);
            // Display register content
            buf.set_string(
                area.left() + 3,
                area.top() + 1 + index as u16,
                content.display(),
                style,
            );
        }
    }
}
