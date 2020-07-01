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

use std::ops::Deref;
use std::time::{Duration, Instant};

use crate::{
    helpers,
    machine::State,
    tui::{input::Input, Backend, SupervisorWrapper, SupervisorWrapperState, Tui},
};

pub const MINIMUM_ALLOWED_WIDTH: u16 = 76;
pub const MINIMUM_ALLOWED_HEIGHT: u16 = 28;
const RIGHT_COLUMN_WIDTH: u16 = 35;
const PROGRAM_AREA_HEIGHT: u16 = 7;
const FREQ_AREA_HEIGHT: u16 = 6;
const INPUT_AREA_HEIGHT: u16 = 2;
const HIGHLIGHT_DURATION: Duration = Duration::from_millis(500);

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
    /// Last displayed frequency.
    measured_frequency: String,
    frequency: String,
}

struct SpacedStr<'l, 'r> {
    left: &'l str,
    right: &'r str,
    left_style: Style,
    right_style: Style,
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
        let measured_frequency = String::from("0Hz");
        let frequency = String::from("0Hz");
        Interface {
            main,
            input,
            program_display,
            freq_display,
            help_display,
            counter,
            measured_frequency,
            frequency,
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
            width: area.width - RIGHT_COLUMN_WIDTH,
            ..area
        };
        f.render_stateful_widget(MainView, main_view_area, tui);

        // This is the area for the right column of the interface.
        // It contains the program, info and help displays.
        let right_column_area = Rect {
            x: area.x + area.width - RIGHT_COLUMN_WIDTH,
            width: RIGHT_COLUMN_WIDTH,
            ..area
        };

        let program_area = Rect {
            height: PROGRAM_AREA_HEIGHT,
            ..right_column_area
        };
        f.render_widget(self.program_display, program_area);
        self.draw_program(f, self.program_display.inner(program_area), tui);

        // Frequency display area
        let freq_area = Rect::new(
            program_area.x,
            program_area.y + program_area.height,
            RIGHT_COLUMN_WIDTH,
            FREQ_AREA_HEIGHT,
        );
        f.render_widget(self.freq_display, freq_area);
        self.draw_freq(f, self.freq_display.inner(freq_area), tui);

        // Help display area
        let help_height = area.height - freq_area.height - program_area.height;
        let help_area = Rect::new(
            freq_area.x,
            freq_area.y + freq_area.height,
            RIGHT_COLUMN_WIDTH,
            help_height,
        );
        f.render_widget(self.help_display, help_area);
        self.draw_help(f, self.help_display.inner(help_area), tui);
    }

    fn draw_help_set(&mut self, f: &mut Frame<Backend>, mut area: Rect) {
        let items = vec![
            ("FC = x", "Input reg FC"),
            ("FD = x", "Input reg FD"),
            ("FE = x", "Input reg FE"),
            ("FF = x", "Input reg FF"),
            ("IRG = x", "MR2DA2 input reg"),
            ("TEMP = x.x", "MR2DA2 Temp voltage"),
            ("I1 = x.x", "MR2DA2 analog input 1"),
            ("I2 = x.x", "MR2DA2 analog input 2"),
            ("J1", "MR2DA2 jumper 1"),
            ("J2", "MR2DA2 jumper 2"),
            ("UIO1", "MR2DA2 universal IO1"),
            ("UIO2", "MR2DA2 universal IO2"),
            ("UIO3", "MR2DA2 universal IO3"),
        ];
        // Show as much as possible
        for (input, help) in items.iter().take(area.height as usize) {
            let ss = SpacedStr::from(input, help);
            f.render_widget(ss, area);
            area.y += 1;
            area.height -= 1;
        }
    }

    fn draw_help_unset(&mut self, f: &mut Frame<Backend>, mut area: Rect) {
        let items = vec![
            ("J1", "MR2DA2 jumper 1"),
            ("J2", "MR2DA2 jumper 2"),
            ("UIO1", "MR2DA2 universal IO1"),
            ("UIO2", "MR2DA2 universal IO2"),
            ("UIO3", "MR2DA2 universal IO3"),
        ];
        // Show as much as possible
        for (input, help) in items.iter().take(area.height as usize) {
            let ss = SpacedStr::from(input, help);
            f.render_widget(ss, area);
            area.y += 1;
            area.height -= 1;
        }
    }

    fn draw_help_keys(&mut self, f: &mut Frame<Backend>, mut area: Rect, tui: &Tui) {
        let items = vec![("Reset", "CTRL+R")];
        let now = Instant::now();
        for (key, help) in items {
            let mut ss = SpacedStr::from(key, help);
            if let Some(ref inst) = tui.last_reset_press {
                if now - *inst < HIGHLIGHT_DURATION {
                    ss = ss.left_style(&helpers::YELLOW);
                }
            }
            f.render_widget(ss, area);
            area.y += 1;
            area.height -= 1;
        }
        let mut ss = SpacedStr::from("Clock", "Enter");
        if let Some(ref inst) = tui.last_clk_press {
            if now - *inst < HIGHLIGHT_DURATION {
                ss = ss.left_style(&helpers::YELLOW);
            }
        }
        if tui.supervisor.machine().state() == State::ErrorStopped
            || tui.supervisor.machine().state() == State::Stopped
            || tui.supervisor.is_auto_run_mode()
        {
            ss = ss
                .left_style(&helpers::DIMMED)
                .right_style(&helpers::DIMMED);
        }
        f.render_widget(ss, area);
        area.y += 1;
        area.height -= 1;
        let mut ss = SpacedStr::from("Edge interrupt", "CTRL+E");
        if let Some(ref inst) = tui.last_int_press {
            if now - *inst < HIGHLIGHT_DURATION {
                ss = ss.left_style(&helpers::YELLOW);
            }
        }
        if !tui.supervisor.machine().is_key_edge_int_enabled() {
            ss = ss
                .left_style(&helpers::DIMMED)
                .right_style(&helpers::DIMMED);
        }
        f.render_widget(ss, area);
        area.y += 1;
        area.height -= 1;
        let mut ss = SpacedStr::from("Toggle autorun", "CTRL+A");
        if tui.supervisor.is_auto_run_mode() {
            ss = ss.left_style(&helpers::YELLOW);
        }
        f.render_widget(ss, area);
        area.y += 1;
        area.height -= 1;
        let mut ss = SpacedStr::from("Toggle asm step", "CTRL+W");
        if tui.supervisor.is_asm_step_mode() {
            ss = ss.left_style(&helpers::YELLOW);
        }
        f.render_widget(ss, area);
        area.y += 1;
        area.height -= 1;
        let mut ss = SpacedStr::from("Continue", "CTRL+L");
        if let Some(ref inst) = tui.last_continue_press {
            if now - *inst < HIGHLIGHT_DURATION {
                ss = ss.left_style(&helpers::YELLOW);
            }
        }
        if tui.supervisor.machine().state() != State::Stopped {
            ss = ss
                .left_style(&helpers::DIMMED)
                .right_style(&helpers::DIMMED);
        }
        f.render_widget(ss, area);

        // Display commands
        area.y += 2;
        area.height -= 2;
        let mut ss = SpacedStr::from("load PATH", "Load asm program");
        if !tui.supervisor.is_program_loaded() {
            ss = ss.left_style(&helpers::YELLOW);
        }
        f.render_widget(ss, area);
        area.y += 1;
        area.height -= 1;
        let ss = SpacedStr::from("set", "Update settings");
        f.render_widget(ss, area);
        area.y += 1;
        area.height -= 1;
        let ss = SpacedStr::from("show", "Select part to display");
        f.render_widget(ss, area);
        area.y += 1;
        area.height -= 1;
        let ss = SpacedStr::from("quit", "Exit the program");
        f.render_widget(ss, area);
    }

    fn draw_help(&mut self, f: &mut Frame<Backend>, area: Rect, tui: &Tui) {
        let input_field_content: String = tui.input_field().current().iter().collect();
        // Draw information about `set` command when entered
        if input_field_content.starts_with("set") {
            self.draw_help_set(f, area);
        } else if input_field_content.starts_with("unset") {
            self.draw_help_unset(f, area);
        } else {
            self.draw_help_keys(f, area, tui);
        }
    }

    fn draw_freq(&mut self, f: &mut Frame<Backend>, mut area: Rect, tui: &Tui) {
        let program_name = match tui.supervisor().get_program_path() {
            Some(program_path) => match program_path.file_name() {
                Some(program_name_os) => program_name_os.to_str().unwrap_or(""),
                None => "",
            },
            None => "",
        };
        let program_ss = SpacedStr::from("Program: ", program_name).left_style(&helpers::DIMMED);
        // Only update the frequency every 100 frames
        if self.counter % 100 == 0 {
            let measured_freq = tui.supervisor.get_measured_frequency();
            self.measured_frequency = helpers::format_number(measured_freq);
            let freq = tui.supervisor.get_frequency();
            self.frequency = helpers::format_number(freq);
        }
        let frequency_measured_ss =
            SpacedStr::from("Measured Frequency: ", &self.measured_frequency)
                .left_style(&helpers::DIMMED);
        let frequency_ss =
            SpacedStr::from("Frequency: ", &self.frequency).left_style(&helpers::DIMMED);
        let mut state_ss = SpacedStr::from("State: ", "RUNNING").left_style(&helpers::DIMMED);
        if tui.supervisor.machine().state() == State::ErrorStopped {
            state_ss.right = "ERROR STOPPED";
            state_ss = state_ss.right_style(&helpers::RED);
        } else if tui.supervisor.machine().state() == State::Stopped {
            state_ss.right = "STOPPED";
            state_ss = state_ss.right_style(&helpers::YELLOW);
        }
        f.render_widget(program_ss, area);
        area.y += 1;
        f.render_widget(frequency_ss, area);
        area.y += 1;
        f.render_widget(frequency_measured_ss, area);
        area.y += 1;
        f.render_widget(state_ss, area);
    }

    fn draw_program(&mut self, f: &mut Frame<Backend>, area: Rect, tui: &Tui) {
        let context = (area.height - 1) / 2;
        let (middle_index, lines) = tui.supervisor.machine().get_current_lines(context as isize);
        let pd = ProgramDisplay::from(middle_index, lines);
        f.render_widget(pd, area);
    }
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
        buf.set_stringn(area.x, area.y, self.left, total_width, self.left_style);
        // Display the right part, if possible
        if left_len + right_len < area.width {
            let right_start = area.x + area.width.saturating_sub(right_len);
            buf.set_string(right_start, area.y, self.right, self.right_style);
        }
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
