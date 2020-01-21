use lazy_static::lazy_static;
use tui::backend::CrosstermBackend;
use tui::buffer::Buffer;
use tui::layout::Alignment;
use tui::layout::Rect;
use tui::style::Modifier;
use tui::style::Style;
use tui::terminal::Frame;
use tui::widgets::Block;
use tui::widgets::Borders;
use tui::widgets::Paragraph;
use tui::widgets::Text;
use tui::widgets::Widget;

use std::ops::Deref;
use std::time::{Duration, Instant};

use crate::helpers;
use crate::machine::State;
use crate::tui::Tui;

pub static MINIMUM_ALLOWED_WIDTH: u16 = 76;
pub static MINIMUM_ALLOWED_HEIGHT: u16 = 25;

const RIGHT_COLUMN_WIDTH: u16 = 35;
const PROGRAM_AREA_HEIGHT: u16 = 7;
const FREQ_AREA_HEIGHT: u16 = 6;
const INPUT_AREA_HEIGHT: u16 = 3;
const HIGHLIGHT_DURATION: Duration = Duration::from_millis(500);
lazy_static! {
    static ref BLK_ERROR: Block<'static> = Block::default()
        .title("Error")
        .borders(Borders::ALL)
        .border_style(*helpers::RED);
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
        let outer = Block::default()
            .title("Minirechner 2a")
            .borders(Borders::ALL);
        let main = Block::default().borders(Borders::ALL);
        let input = Block::default()
            .borders(Borders::ALL)
            .border_style(*helpers::YELLOW);
        let program_display = Block::default().borders(Borders::ALL);
        let freq_display = Block::default().borders(Borders::ALL);
        let help_display = Block::default().borders(Borders::ALL).title("HELP");
        let counter = 0;
        let measured_frequency = String::from("0Hz");
        let frequency = String::from("0Hz");
        Interface {
            outer,
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
    pub fn draw<'b>(&mut self, tui: &'b mut Tui, f: &mut Frame<CrosstermBackend>) {
        self.counter = self.counter.overflowing_add(1).0;
        let area = f.size();
        let area = Rect::new(area.x, area.y, area.width - 1, area.height - 1);
        // Draw a placeholder for too small windows
        if area.width < MINIMUM_ALLOWED_WIDTH {
            let test = [
                Text::raw("Terminal width too small!\n Please"),
                Text::styled(" resize your terminal", *helpers::YELLOW),
                Text::raw(" or"),
                Text::styled(" decrease your font size", *helpers::YELLOW),
                Text::raw("!"),
            ];
            let mut paragraph = Paragraph::new(test.iter())
                .alignment(Alignment::Center)
                .block(*BLK_ERROR);
            paragraph.render(f, area);
            return;
        } else if area.height < MINIMUM_ALLOWED_HEIGHT {
            let test = [
                Text::raw("Terminal height too small!\n Please"),
                Text::styled(" resize your terminal", *helpers::YELLOW),
                Text::raw(" or"),
                Text::styled(" decrease your font size", *helpers::YELLOW),
                Text::raw("!"),
            ];
            let mut paragraph = Paragraph::new(test.iter())
                .alignment(Alignment::Center)
                .block(*BLK_ERROR);
            paragraph.render(f, area);
            return;
        }

        // Outer area
        self.outer.render(f, area);

        // Machine area (main)
        let mut main_area = self.outer.inner(area);
        main_area.height -= INPUT_AREA_HEIGHT;
        main_area.width -= RIGHT_COLUMN_WIDTH;
        self.main.render(f, main_area);
        tui.supervisor.render(f, self.main.inner(main_area));

        // Input area
        let input_area = Rect::new(
            main_area.x,
            main_area.y + main_area.height,
            main_area.width,
            INPUT_AREA_HEIGHT,
        );
        self.input.render(f, input_area);
        tui.input_field.render(f, self.input.inner(input_area));

        // Program display area
        let program_area = Rect::new(
            main_area.x + main_area.width,
            main_area.y,
            RIGHT_COLUMN_WIDTH,
            PROGRAM_AREA_HEIGHT,
        );
        self.program_display.render(f, program_area);
        self.draw_program(f, self.program_display.inner(program_area), tui);

        // Frequency display area
        let freq_area = Rect::new(
            program_area.x,
            program_area.y + program_area.height,
            RIGHT_COLUMN_WIDTH,
            FREQ_AREA_HEIGHT,
        );
        self.freq_display.render(f, freq_area);
        self.draw_freq(f, self.freq_display.inner(freq_area), tui);

        // Help display area
        let help_height = self.outer.inner(area).height - freq_area.height - program_area.height;
        let help_area = Rect::new(
            freq_area.x,
            freq_area.y + freq_area.height,
            RIGHT_COLUMN_WIDTH,
            help_height,
        );
        self.help_display.render(f, help_area);
        self.draw_help(f, self.help_display.inner(help_area), tui);
    }

    fn draw_help_set(&mut self, f: &mut Frame<CrosstermBackend>, mut area: Rect) {
        let items = vec![
            ("FC = x", "Input reg FC"),
            ("FD = x", "Input reg FD"),
            ("FE = x", "Input reg FE"),
            ("FD = x", "Input reg FF"),
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
            let mut ss = SpacedStr::from(input, help);
            ss.render(f, area);
            area.y += 1;
            area.height -= 1;
        }
    }

    fn draw_help_unset(&mut self, f: &mut Frame<CrosstermBackend>, mut area: Rect) {
        let items = vec![
            ("J1", "MR2DA2 jumper 1"),
            ("J2", "MR2DA2 jumper 2"),
            ("UIO1", "MR2DA2 universal IO1"),
            ("UIO2", "MR2DA2 universal IO2"),
            ("UIO3", "MR2DA2 universal IO3"),
        ];
        // Show as much as possible
        for (input, help) in items.iter().take(area.height as usize) {
            let mut ss = SpacedStr::from(input, help);
            ss.render(f, area);
            area.y += 1;
            area.height -= 1;
        }
    }

    fn draw_help_keys(&mut self, f: &mut Frame<CrosstermBackend>, mut area: Rect, tui: &Tui) {
        let items = vec![("Reset", "CTRL+R")];
        let now = Instant::now();
        for (key, help) in items {
            let mut ss = SpacedStr::from(key, help);
            if let Some(ref inst) = tui.last_reset_press {
                if now - *inst < HIGHLIGHT_DURATION {
                    ss = ss.left_style(&helpers::YELLOW);
                }
            }
            ss.render(f, area);
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
        ss.render(f, area);
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
        ss.render(f, area);
        area.y += 1;
        area.height -= 1;
        let mut ss = SpacedStr::from("Toggle autorun", "CTRL+A");
        if tui.supervisor.is_auto_run_mode() {
            ss = ss.left_style(&helpers::YELLOW);
        }
        ss.render(f, area);
        area.y += 1;
        area.height -= 1;
        let mut ss = SpacedStr::from("Toggle asm step", "CTRL+W");
        if tui.supervisor.is_asm_step_mode() {
            ss = ss.left_style(&helpers::YELLOW);
        }
        ss.render(f, area);
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
        ss.render(f, area);

        // Display commands
        area.y += 2;
        area.height -= 2;
        let mut ss = SpacedStr::from("load PATH", "Load asm program");
        if !tui.supervisor.is_program_loaded() {
            ss = ss.left_style(&helpers::YELLOW);
        }
        ss.render(f, area);
        area.y += 1;
        area.height -= 1;
        let mut ss = SpacedStr::from("set", "Update settings");
        ss.render(f, area);
        area.y += 1;
        area.height -= 1;
        let mut ss = SpacedStr::from("show", "Select part to display");
        ss.render(f, area);
        area.y += 1;
        area.height -= 1;
        let mut ss = SpacedStr::from("quit", "Exit the program");
        ss.render(f, area);
    }

    fn draw_help(&mut self, f: &mut Frame<CrosstermBackend>, area: Rect, tui: &Tui) {
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

    fn draw_freq(&mut self, f: &mut Frame<CrosstermBackend>, mut area: Rect, tui: &Tui) {
        let program_name = match tui.supervisor().get_program_path() {
            Some(program_path) => match program_path.file_name() {
                Some(program_name_os) => program_name_os.to_str().unwrap_or(""),
                None => "",
            },
            None => "",
        };
        let mut program_ss =
            SpacedStr::from("Program: ", program_name).left_style(&helpers::DIMMED);
        // Only update the frequency every 100 frames
        if self.counter % 100 == 0 {
            let measured_freq = tui.supervisor.get_measured_frequency();
            self.measured_frequency = helpers::format_number(measured_freq);
            let freq = tui.supervisor.get_frequency();
            self.frequency = helpers::format_number(freq);
        }
        let mut frequency_measured_ss =
            SpacedStr::from("Measured Frequency: ", &self.measured_frequency)
                .left_style(&helpers::DIMMED);
        let mut frequency_ss =
            SpacedStr::from("Frequency: ", &self.frequency).left_style(&helpers::DIMMED);
        let mut state_ss = SpacedStr::from("State: ", "RUNNING").left_style(&helpers::DIMMED);
        if tui.supervisor.machine().state() == State::ErrorStopped {
            state_ss.right = "ERROR STOPPED";
            state_ss = state_ss.right_style(&helpers::RED);
        } else if tui.supervisor.machine().state() == State::Stopped {
            state_ss.right = "STOPPED";
            state_ss = state_ss.right_style(&helpers::YELLOW);
        }
        program_ss.render(f, area);
        area.y += 1;
        frequency_ss.render(f, area);
        area.y += 1;
        frequency_measured_ss.render(f, area);
        area.y += 1;
        state_ss.render(f, area);
    }

    fn draw_program(&mut self, f: &mut Frame<CrosstermBackend>, area: Rect, tui: &Tui) {
        let context = (area.height - 1) / 2;
        let (middle_index, lines) = tui.supervisor.machine().get_current_lines(context as isize);
        let mut pd = ProgramDisplay::from(middle_index, lines);
        pd.render(f, area);
    }
}

impl<'l, 'r> SpacedStr<'l, 'r> {
    /// Create a spaced string from two strings.
    pub fn from(left: &'l str, right: &'r str) -> Self {
        SpacedStr {
            left: left,
            right: right,
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
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
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
    fn draw(&mut self, mut area: Rect, buf: &mut Buffer) {
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
                Some(line) => buf.set_stringn(
                    area.x,
                    area.y + i as u16,
                    line,
                    area.width as usize,
                    if i == middle {
                        Style::default()
                    } else {
                        dimmed
                    },
                ),
                _ => buf.set_string(area.x, area.y + i as u16, &empty, Style::default()),
            }
        }
    }
}
