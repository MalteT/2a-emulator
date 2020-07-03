use tui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{StatefulWidget, Widget},
};
use unicode_width::UnicodeWidthStr;

use std::ops::Deref;

mod command_help;
mod keybinding_help;
mod program_display;
mod program_info;

use crate::{helpers, machine::RegisterNumber, tui::Tui};
pub use command_help::CommandHelpWidget;
pub use keybinding_help::{KeybindingHelpState, KeybindingHelpWidget};
pub use program_display::{ProgramDisplayState, ProgramDisplayWidget};
pub use program_info::ProgramInfoWidget;

pub const HEADER_HEIGHT: u16 = 1;

pub struct ProgramHelpSidebar;

impl StatefulWidget for ProgramHelpSidebar {
    /// Input registers FC, FD, FE, FF.
    type State = Tui;

    fn render(self, mut area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Render the bottom line
        buf.set_string(
            area.left(),
            area.bottom().saturating_sub(1),
            "─".repeat(area.width as usize),
            *helpers::DIMMED,
        );
        area.height -= 1;
        // Render the command help widget right at the bottom
        let current_input: String = state.input_field.current().iter().collect();
        let command_help_height = CommandHelpWidget::calculate_height(&current_input);
        let command_help_area = Rect {
            y: area.bottom() - command_help_height,
            height: command_help_height,
            ..area
        };
        CommandHelpWidget(&current_input).render(command_help_area, buf);
        area.height -= command_help_height;
        // Render the keybindings help
        let keybinding_help_height = KeybindingHelpWidget::calculate_height();
        let keybinding_help_area = Rect {
            y: area.bottom() - keybinding_help_height,
            height: keybinding_help_height,
            ..area
        };
        KeybindingHelpWidget.render(keybinding_help_area, buf, &mut state.keybinding_state);
        area.height -= keybinding_help_height;
        // Render the info widget right at the top
        let info_height = ProgramInfoWidget::calculate_height();
        let info_area = Rect {
            height: info_height,
            ..area
        };
        ProgramInfoWidget::from(state).render(info_area, buf);
        area.y += info_height;
        area.height -= info_height;
        // The rest of the area can be used for the program display
        let program_display_area = area;
        ProgramDisplayWidget(
            state
                .supervisor()
                .machine()
                .registers()
                .get(RegisterNumber::R3),
        )
        .render(program_display_area, buf, &mut state.program_display_state);
    }
}

struct SpacedStr<'l, 'r> {
    left: &'l str,
    right: &'r str,
    left_style: Style,
    right_style: Style,
}

impl<'l, 'r> SpacedStr<'l, 'r> {
    /// Create a spaced string from two strings.
    pub fn from(left: &'l str, right: &'r str) -> Self {
        SpacedStr {
            left,
            right,
            left_style: Style::default(),
            right_style: Style::default(),
        }
    }
    /// Set the left style.
    pub fn left_style<S: Deref<Target = Style>>(mut self, style: &S) -> Self {
        self.left_style = **style;
        self
    }
    /// Set the right style.
    pub fn right_style<S: Deref<Target = Style>>(mut self, style: &S) -> Self {
        self.right_style = **style;
        self
    }
}

impl Widget for SpacedStr<'_, '_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let total_width = area.width as usize;
        let left_len = self.left.len() as u16;
        let right_len = self.right.len() as u16;
        // Always display as much of the left part as possible.
        buf.set_stringn(
            area.left(),
            area.top(),
            self.left,
            total_width,
            self.left_style,
        );
        // Display the right part, if possible
        if left_len + right_len <= area.width {
            let right_start = area.right().saturating_sub(right_len).saturating_sub(1);
            buf.set_string(right_start, area.y, self.right, self.right_style);
        }
    }
}

fn make_header(title: &str, width: u16) -> String {
    let mut ret = String::from("━╸");
    ret += title;
    ret += "╺";
    ret += &"━".repeat((width as usize).saturating_sub(ret.width()));
    ret
}
