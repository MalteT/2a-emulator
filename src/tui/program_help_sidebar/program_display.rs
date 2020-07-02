use tui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Borders, Widget},
};

use crate::helpers;

pub struct ProgramDisplayWidget;

impl Widget for ProgramDisplayWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let header = super::make_header("Program", area.width);
        buf.set_string(area.left(), area.top(), header, *helpers::DIMMED_BOLD);
        // TODO: Show the assembler program
        let area = Rect {
            y: area.y + 1,
            height: area.height - 1,
            ..area
        };
        Block::default()
            .borders(Borders::ALL)
            .border_style(*helpers::YELLOW_BOLD)
            .render(area, buf);
        buf.set_string(
            area.left() + 2,
            area.top() + 2,
            "TODO!",
            *helpers::YELLOW_BOLD,
        );
    }
}
