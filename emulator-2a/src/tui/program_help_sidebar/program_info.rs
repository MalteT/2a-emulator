//! Everything related to drawing the [`ProgramInfoWidget`].
use emulator_2a_lib::machine::State;
use tui::{buffer::Buffer, layout::Rect, widgets::Widget};

use std::{borrow::Cow, path::PathBuf};

use super::{SpacedStr, HEADER_HEIGHT};
use crate::{helpers, tui::Tui};

const WIDGET_HEIGHT: u16 = 4 + HEADER_HEIGHT;
const INFO_PROGRAM: (&str, &str) = ("Program:", "");
const INFO_FREQ: (&str, &str) = ("Frequency:", "");
const INFO_FREQ_MEASURED: (&str, &str) = ("Measured Frequency:", "");
const INFO_STATE: (&str, &str) = ("State:", "");

/// Widget for additional information about the
/// current execution.
///
/// # Example
///
/// ```text
/// ━╸Info╺━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// Program:     11-simple-addition.asm
/// Frequency:                  7.41MHz
/// Measured Frequency:          0.00Hz
/// State:                      Running
/// ```
pub struct ProgramInfoWidget<'a> {
    program: Option<&'a PathBuf>,
    freq: f32,
    freq_measured: f32,
    state: State,
}

impl<'a> ProgramInfoWidget<'a> {
    /// Read all necessary information from the given [`Tui`].
    pub fn from(tui: &'a Tui) -> Self {
        let program = tui.machine.program_path();
        let freq = tui.machine.get_frequency();
        let freq_measured = tui.machine.get_measured_frequency();
        let state = tui.machine.state();
        ProgramInfoWidget {
            program,
            freq,
            freq_measured,
            state,
        }
    }
    /// Get the height necessary for drawing this widget.
    pub fn calculate_height() -> u16 {
        WIDGET_HEIGHT
    }
    fn render_program(&self, area: Rect, buf: &mut Buffer) {
        let name = self
            .program
            .as_ref()
            .and_then(|path| path.file_name())
            .map(|file_name| file_name.to_string_lossy())
            .unwrap_or(Cow::Borrowed(""));
        let spaced = SpacedStr::from(INFO_PROGRAM.0, &name);
        spaced.render(area, buf)
    }
    fn render_freq(&self, area: Rect, buf: &mut Buffer) {
        let freq = helpers::format_number(self.freq);
        let spaced = SpacedStr::from(INFO_FREQ.0, &freq);
        spaced.render(area, buf)
    }
    fn render_freq_measured(&self, area: Rect, buf: &mut Buffer) {
        let freq_measured = helpers::format_number(self.freq_measured);
        let spaced = SpacedStr::from(INFO_FREQ_MEASURED.0, &freq_measured);
        spaced.render(area, buf)
    }
    fn render_state(&self, area: Rect, buf: &mut Buffer) {
        let spaced = match self.state {
            State::Running => SpacedStr::from(INFO_STATE.0, "Running"),
            State::Stopped => {
                SpacedStr::from(INFO_STATE.0, "Stopped").right_style(&helpers::YELLOW_BOLD)
            }
            State::ErrorStopped => {
                SpacedStr::from(INFO_STATE.0, "Stopped by Error").right_style(&helpers::RED_BOLD)
            }
        };
        spaced.render(area, buf)
    }
}

impl<'a> Widget for ProgramInfoWidget<'a> {
    fn render(self, mut area: Rect, buf: &mut Buffer) {
        // Render header
        let header = super::make_header("Info", area.width);
        buf.set_string(area.left(), area.top(), header, *helpers::DIMMED_BOLD);
        area.y += 1;
        area.height -= 1;
        self.render_program(area, buf);
        area.y += 1;
        area.height -= 1;
        self.render_freq(area, buf);
        area.y += 1;
        area.height -= 1;
        self.render_freq_measured(area, buf);
        area.y += 1;
        area.height -= 1;
        self.render_state(area, buf);
    }
}
