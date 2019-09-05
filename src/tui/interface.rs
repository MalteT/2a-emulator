use lazy_static::lazy_static;
use tui::backend::CrosstermBackend;
use tui::layout::Rect;
use tui::terminal::Frame;
use tui::widgets::Block;
use tui::widgets::Borders;
use tui::widgets::Widget;
use tui::buffer::Buffer;
use tui::style::Style;

use std::ops::Deref;

use super::input::Input;
use crate::schematic::Machine;

lazy_static! {
    static ref RIGHT_COLUMN_WIDTH: u16 = 35;
    static ref PROGRAM_AREA_HEIGHT: u16 = 7;
    static ref FREQ_AREA_HEIGHT: u16 = 3;
    static ref INPUT_AREA_HEIGHT: u16 = 3;
}

/// The user interface.
/// ```text
///            ┌──────────────────────────────────────┬────────────────────────┐
///          ^ │                                      │                        │ ^
///          | │ R0 ○●○○○○○○                          │   04: SOME_LABEL:      │ |
///          | │ R1 ○○●●○○○○                          │ > 05: MV 0xFF, (R2)    │ | fixed height
/// min.     | │ R2 ○○○○○○○○                          │   06: MV R1, (R2)      │ |
/// height   │ | SP ○○○○●○●●                          │                        │ v
///          | │                                      ├────────────────────────┤
///          | │                                      │ 777 Hz     asmfile.asm │ < fixed height
///          | │                                      ├────────────────────────┤
///          | │                                      │                        │
///          | │                                      │ .                 step │
///          | │ ○○○○○○○● ○○○○○○●● ○○○○○○○○ ○○○○○○○○  │ Enter              run │
///          | │  in  FF   in  FE   in  FD   in  FC   │                        │
///          | │                                      │                        │
///          | │ ○○○○○○○○ ○○●●●○●○                    │                        │
///          | │  out FF   out FE                     │                        │
///          v │                                      │                        │
///            └──────────────────────────────────────┴────────────────────────┘
///             <------------------------------------> <---------------------->
///                        minimal width                      fixed width
/// ```
pub struct Interface<'a> {
    pub outer: Block<'a>,
    pub main: Block<'a>,
    pub input: Block<'a>,
    pub program_display: Block<'a>,
    pub freq_display: Block<'a>,
    pub help_display: Block<'a>,
}

struct SpacedString {
    left: String,
    right: String,
}

impl<'a> Interface<'a> {
    /// Initialize a new interface.
    pub fn new() -> Self {
        let outer = Block::default()
            .title("Minirechner 2a")
            .borders(Borders::ALL);
        let main = Block::default().borders(Borders::ALL);
        let input = Block::default().borders(Borders::ALL);
        let program_display = Block::default().borders(Borders::ALL);
        let freq_display = Block::default().borders(Borders::ALL);
        let help_display = Block::default().borders(Borders::ALL).title("HELP");
        Interface {
            outer,
            main,
            input,
            program_display,
            freq_display,
            help_display,
        }
    }
    /// Draw the interface using the given machine and frame.
    pub fn draw(
        &mut self,
        machine: &mut Machine,
        input_field: &mut Input,
        f: &mut Frame<CrosstermBackend>,
    ) {
        let area = f.size();

        // Outer area
        self.outer.render(f, area);

        // Machine area (main)
        let mut main_area = self.outer.inner(area);
        main_area.height -= *INPUT_AREA_HEIGHT.deref();
        main_area.width -= *RIGHT_COLUMN_WIDTH.deref();
        self.main.render(f, main_area);
        machine.render(f, self.main.inner(main_area));

        // Input area
        let input_area = Rect::new(
            main_area.x,
            main_area.y + main_area.height,
            main_area.width,
            *INPUT_AREA_HEIGHT.deref(),
        );
        self.input.render(f, input_area);
        input_field.render(f, self.input.inner(input_area));

        // Program display area
        let program_area = Rect::new(
            main_area.x + main_area.width,
            main_area.y,
            *RIGHT_COLUMN_WIDTH.deref(),
            *PROGRAM_AREA_HEIGHT.deref(),
        );
        self.program_display.render(f, program_area);

        // Frequency display area
        let freq_area = Rect::new(
            program_area.x,
            program_area.y + program_area.height,
            *RIGHT_COLUMN_WIDTH.deref(),
            *FREQ_AREA_HEIGHT.deref(),
        );
        self.freq_display.render(f, freq_area);

        // Help display area
        let help_height = self.outer.inner(area).height - freq_area.height - program_area.height;
        let help_area = Rect::new(
            freq_area.x,
            freq_area.y + freq_area.height,
            *RIGHT_COLUMN_WIDTH.deref(),
            help_height,
        );
        self.help_display.render(f, help_area);
        self.draw_help(f, self.help_display.inner(help_area));
    }

    pub fn draw_help(&mut self, f: &mut Frame<CrosstermBackend>, mut area: Rect) {
        let items = vec![("CTRL+A", "Toggle autorun"), ("Enter", "Clock")];
        for (key, help) in items {
            let mut ss = SpacedString::from(key, help);
            ss.render(f, area);
            area.y += 1;
            area.height -= 1;
        }
    }
}

impl SpacedString {
    fn from(left: &'static str, right: &'static str) -> Self {
        SpacedString {
            left: left.into(),
            right: right.into(),
        }
    }
}

impl Widget for SpacedString {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
       let width = area.width - self.right.len() as u16;
       let s = format!("{0:width$}{1}", self.left, self.right, width = width as usize);
       buf.set_stringn(area.x, area.y, s, area.width as usize, Style::default());
    }
}
