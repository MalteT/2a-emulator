//! The basis of the TUI user interface.
//! ┌Minirechner 2a────────────────────────┐ASM──────────────────────────────
//! │                                      │
//! │ Outputs:                             │
//! │ 00000000 00000000                    │>
//! │       FF       FE                    │
//! │                                      │
//! │ Inputs:                              │INFO─────────────────────────────
//! │ 00000000 00000000 00000000 00000000  │
//! │       FF       FE       FD       FC  │Program:
//! │                                      │Frequency:                7.41MHz
//! │ Registers:                           │Measured Frequency:        0.00Hz
//! │ R0 00000000                          │State:                    RUNNING
//! │ R1 00000000                          │HELP─────────────────────────────
//! │ R2 00000000                          │
//! │ PC 00000000                          │Reset                      CTRL+R
//! │ FR 00000000                          │Clock                       Enter
//! │ SP 00000000                          │Edge interrupt             CTRL+E
//! │ R6 00000000                          │Toggle autorun             CTRL+A
//! │ R7 00000000                          │Toggle asm step            CTRL+W
//! │                                      │Continue                   CTRL+L
//! │                                      │
//! │                                      │load PATH        Load asm program
//! │                                      │set               Update settings
//! │                                      │show       Select part to display
//! │──────────────────────────────────────│quit             Exit the program
//! │> █                                   │
//! └──────────────────────────────────────┘─────────────────────────────────

use lazy_static::lazy_static;
use tui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    terminal::Frame,
    widgets::{Block, Borders, Paragraph, StatefulWidget, Text, Widget},
};

use crate::{
    helpers,
    tui::{input::Input, Backend, ProgramHelpSidebar, SupervisorWrapper, Tui},
};

pub const MINIMUM_ALLOWED_WIDTH: u16 = 76;
pub const MINIMUM_ALLOWED_HEIGHT: u16 = 28;
const RIGHT_SIDEBAR_WIDTH: u16 = 35;
const PROGRAM_AREA_HEIGHT: u16 = 7;
const INPUT_AREA_HEIGHT: u16 = 2;

lazy_static! {
    static ref BLK_ERROR: Block<'static> = Block::default()
        .title("Error")
        .borders(Borders::ALL)
        .border_style(*helpers::RED);
}

#[derive(Debug, Clone, Copy)]
struct MainView;

impl StatefulWidget for MainView {
    type State = Tui;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Render outside block
        let main_block = Block::default()
            .border_style(*helpers::DIMMED)
            .title_style(*helpers::DIMMED)
            .title("─┤ Minirechner 2a ├")
            .borders(Borders::ALL);
        main_block.render(area, buf);
        let area_inside_block = main_block.inner(area);

        // Render the input field
        let input_area = Rect {
            y: area_inside_block.bottom() - INPUT_AREA_HEIGHT,
            height: INPUT_AREA_HEIGHT,
            ..area_inside_block
        };
        // Draw the input block first
        let input_block = Block::default()
            .border_style(*helpers::DIMMED)
            .borders(Borders::TOP);
        input_block.render(input_area, buf);
        // Then render the actual input field
        let input_field_area = input_block.inner(input_area);
        Input::new().render(input_field_area, buf, &mut state.input_field);
        // Render the rest of the main view, registers and the shown part.
        let main_machine_area = Rect {
            height: area_inside_block.height - INPUT_AREA_HEIGHT,
            ..area_inside_block
        };
        SupervisorWrapper::new().render(main_machine_area, buf, &mut state.supervisor);
    }
}

/// The user interface.
/// ```text
///            ┌──────────────────────────────────────┬────────────────────────┐
///          ^ │                                      │                        │ ^
///          | │ R0 ○●○○○○○○                          │   04: SOME_LABEL:      │ |
///          | │ R1 ○○●●○○○○                          │ > 05: MV 0xFF, (R2)    │ | fixed
/// min.     | │ R2 ○○○○○○○○                          │   06: MV R1, (R2)      │ | height
/// height   │ | SP ○○○○●○●●                          │                        │ v
///          | │                                      ├────────────────────────┤
///          | │                                      │ 777 Hz     asmfile.asm │ < fixed
///          | │                                      ├────────────────────────┤   height
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
    pub main: Block<'a>,
    pub input: Block<'a>,
    pub program_display: Block<'a>,
    pub freq_display: Block<'a>,
    pub help_display: Block<'a>,
    /// For updating some things not every frame.
    counter: usize,
}

struct ProgramDisplay<'a> {
    lines: Vec<&'a String>,
    middle_index: usize,
}

impl<'a> Interface<'a> {
    /// Initialize a new interface.
    pub fn new() -> Self {
        let main = Block::default().borders(Borders::ALL);
        let input = Block::default()
            .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
            .border_style(*helpers::YELLOW);
        let program_display = Block::default()
            .borders(Borders::TOP)
            .border_style(*helpers::DIMMED)
            .title_style(*helpers::DIMMED)
            .title("─┤ Program ├");
        let freq_display = Block::default()
            .borders(Borders::TOP)
            .border_style(*helpers::DIMMED)
            .title_style(*helpers::DIMMED)
            .title("─┤ Info    ├");
        let help_display = Block::default()
            .borders(Borders::TOP)
            .border_style(*helpers::DIMMED)
            .title_style(*helpers::DIMMED)
            .title("─┤ Help    ├");
        let counter = 0;
        Interface {
            main,
            input,
            program_display,
            freq_display,
            help_display,
            counter,
        }
    }
    /// Draw the interface using information from the given [`Tui`]
    pub fn draw<'b>(&mut self, tui: &'b mut Tui, f: &mut Frame<Backend>) {
        // Increment draw counter
        self.counter = self.counter.overflowing_add(1).0;
        let area = f.size();
        let area = Rect::new(area.x, area.y, area.width - 1, area.height);
        // Draw a placeholder for too small windows
        if area.width < MINIMUM_ALLOWED_WIDTH {
            let test = [
                Text::raw("Terminal width too small!\n Please"),
                Text::styled(" resize your terminal", *helpers::YELLOW),
                Text::raw(" or"),
                Text::styled(" decrease your font size", *helpers::YELLOW),
                Text::raw("!"),
            ];
            let paragraph = Paragraph::new(test.iter())
                .alignment(Alignment::Center)
                .block(*BLK_ERROR);
            f.render_widget(paragraph, area);
            return;
        } else if area.height < MINIMUM_ALLOWED_HEIGHT {
            let test = [
                Text::raw("Terminal height too small!\n Please"),
                Text::styled(" resize your terminal", *helpers::YELLOW),
                Text::raw(" or"),
                Text::styled(" decrease your font size", *helpers::YELLOW),
                Text::raw("!"),
            ];
            let paragraph = Paragraph::new(test.iter())
                .alignment(Alignment::Center)
                .block(*BLK_ERROR);
            f.render_widget(paragraph, area);
            return;
        }

        // This is the area for the main component, the [`MainView`].
        let main_view_area = Rect {
            width: area.width - RIGHT_SIDEBAR_WIDTH,
            ..area
        };
        f.render_stateful_widget(MainView, main_view_area, tui);

        // This is the area for the sidebar on the right, the [`ProgramHelpSidebar`].
        let help_area = Rect {
            x: area.right() - RIGHT_SIDEBAR_WIDTH,
            width: RIGHT_SIDEBAR_WIDTH,
            ..area
        };
        f.render_stateful_widget(ProgramHelpSidebar, help_area, tui);

        // This is the area for the right column of the interface.
        // It contains the program, info and help displays.
        let right_column_area = Rect {
            x: area.x + area.width - RIGHT_SIDEBAR_WIDTH,
            width: RIGHT_SIDEBAR_WIDTH,
            ..area
        };

        let program_area = Rect {
            height: PROGRAM_AREA_HEIGHT,
            ..right_column_area
        };
        // f.render_widget(self.program_display, program_area);
        // self.draw_program(f, self.program_display.inner(program_area), tui);
    }

    fn draw_program(&mut self, f: &mut Frame<Backend>, area: Rect, tui: &Tui) {
        let context = (area.height - 1) / 2;
        let (middle_index, lines) = tui.supervisor.machine().get_current_lines(context as isize);
        let pd = ProgramDisplay::from(middle_index, lines);
        f.render_widget(pd, area);
    }
}

impl<'a> ProgramDisplay<'a> {
    fn from(middle_index: usize, lines: Vec<&'a String>) -> Self {
        ProgramDisplay {
            middle_index,
            lines,
        }
    }
}

impl Widget for ProgramDisplay<'_> {
    fn render(self, mut area: Rect, buf: &mut Buffer) {
        let middle = area.height as i16 / 2;
        let dimmed = Style::default().modifier(Modifier::DIM);
        buf.set_string(area.x, area.y + middle as u16, ">", Style::default());
        // Move everything two to the left to leave space for the arrow.
        area.x += 2;
        area.width -= 2;
        // If machine stopped show the red sign
        let empty = " ".repeat(area.width as usize);
        for i in 0..area.height as i16 {
            let index = self.middle_index as i16 + i - middle;
            match self.lines.get(index as usize) {
                Some(line) => {
                    buf.set_stringn(
                        area.x,
                        area.y + i as u16,
                        line,
                        area.width as usize,
                        if i == middle {
                            Style::default()
                        } else {
                            dimmed
                        },
                    );
                }
                _ => buf.set_string(area.x, area.y + i as u16, &empty, Style::default()),
            }
        }
    }
}
