use tui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{StatefulWidget, Widget},
};

use std::time::{Duration, Instant};

use super::{SpacedStr, HEADER_HEIGHT};
use crate::helpers;

const WIDGET_HEIGHT: u16 = 6 + HEADER_HEIGHT;
const HIGHLIGHT_DURATION: Duration = Duration::from_millis(500);
const BIND_CLK: (&str, &str) = ("Clock", "Enter");
const BIND_TOGGLE_AUTORUN: (&str, &str) = ("Toggle autorun", "CTRL+A");
const BIND_TOGGLE_ASM_STEP: (&str, &str) = ("Toggle asm step", "CTRL+W");
const BIND_RESET: (&str, &str) = ("Reset", "CTRL+R");
const BIND_EDGE_INT: (&str, &str) = ("Edge interrupt", "CTRL+E");
const BIND_CONTINUE: (&str, &str) = ("Continue", "CTRL+L");

pub struct KeybindingHelpWidget;

impl KeybindingHelpWidget {
    pub fn calculate_height() -> u16 {
        // One line for each binding + one for the header
        WIDGET_HEIGHT
    }
    fn render_clk(area: Rect, buf: &mut Buffer, state: &mut KeybindingHelpState) {
        let mut spaced = SpacedStr::from(BIND_CLK.0, BIND_CLK.1);
        if is_within_highlight_dur(&mut state.last_clk_press) {
            spaced = spaced.left_style(&helpers::BOLD);
        }
        spaced.render(area, buf)
    }
    fn render_toggle_autorun(area: Rect, buf: &mut Buffer, state: &mut KeybindingHelpState) {
        let mut spaced = SpacedStr::from(BIND_TOGGLE_AUTORUN.0, BIND_TOGGLE_AUTORUN.1);
        if state.is_autorun_on {
            spaced = spaced.left_style(&helpers::BOLD);
        }
        spaced.render(area, buf)
    }
    fn render_toggle_asm_step(area: Rect, buf: &mut Buffer, state: &mut KeybindingHelpState) {
        let mut spaced = SpacedStr::from(BIND_TOGGLE_ASM_STEP.0, BIND_TOGGLE_ASM_STEP.1);
        if state.is_asm_step_on {
            spaced = spaced.left_style(&helpers::BOLD);
        }
        spaced.render(area, buf)
    }
    fn render_reset(area: Rect, buf: &mut Buffer, state: &mut KeybindingHelpState) {
        let mut spaced = SpacedStr::from(BIND_RESET.0, BIND_RESET.1);
        if is_within_highlight_dur(&mut state.last_reset_press) {
            spaced = spaced.left_style(&helpers::BOLD);
        }
        spaced.render(area, buf)
    }
    fn render_edge_int(area: Rect, buf: &mut Buffer, state: &mut KeybindingHelpState) {
        let mut spaced = SpacedStr::from(BIND_EDGE_INT.0, BIND_EDGE_INT.1);
        if is_within_highlight_dur(&mut state.last_edge_int_press) {
            spaced = spaced.left_style(&helpers::BOLD);
        } else if !state.is_edge_int_possible {
            spaced = spaced
                .left_style(&helpers::DIMMED)
                .right_style(&helpers::DIMMED);
        }
        spaced.render(area, buf)
    }
    fn render_continue(area: Rect, buf: &mut Buffer, state: &mut KeybindingHelpState) {
        let mut spaced = SpacedStr::from(BIND_CONTINUE.0, BIND_CONTINUE.1);
        if is_within_highlight_dur(&mut state.last_continue_press) {
            spaced = spaced.left_style(&helpers::BOLD);
        } else if !state.is_continue_possible {
            spaced = spaced
                .left_style(&helpers::DIMMED)
                .right_style(&helpers::DIMMED);
        }
        spaced.render(area, buf)
    }
}

impl StatefulWidget for KeybindingHelpWidget {
    type State = KeybindingHelpState;

    fn render(self, mut area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Render header
        let header = super::make_header("Keybindings", area.width);
        buf.set_string(area.left(), area.top(), header, *helpers::DIMMED_BOLD);
        area.y += 1;
        area.height -= 1;
        KeybindingHelpWidget::render_clk(area, buf, state);
        area.y += 1;
        area.height -= 1;
        KeybindingHelpWidget::render_toggle_autorun(area, buf, state);
        area.y += 1;
        area.height -= 1;
        KeybindingHelpWidget::render_toggle_asm_step(area, buf, state);
        area.y += 1;
        area.height -= 1;
        KeybindingHelpWidget::render_reset(area, buf, state);
        area.y += 1;
        area.height -= 1;
        KeybindingHelpWidget::render_edge_int(area, buf, state);
        area.y += 1;
        area.height -= 1;
        KeybindingHelpWidget::render_continue(area, buf, state);
    }
}

pub struct KeybindingHelpState {
    last_clk_press: Option<Instant>,
    last_reset_press: Option<Instant>,
    last_edge_int_press: Option<Instant>,
    last_continue_press: Option<Instant>,
    is_autorun_on: bool,
    is_asm_step_on: bool,
    is_edge_int_possible: bool,
    is_continue_possible: bool,
}

impl KeybindingHelpState {
    pub const fn init() -> Self {
        KeybindingHelpState {
            last_clk_press: None,
            last_reset_press: None,
            last_edge_int_press: None,
            last_continue_press: None,
            is_autorun_on: false,
            is_asm_step_on: false,
            is_edge_int_possible: false,
            is_continue_possible: false,
        }
    }
    pub fn clk_pressed(&mut self) {
        self.last_clk_press = Some(Instant::now());
    }
    pub fn reset_pressed(&mut self) {
        self.last_reset_press = Some(Instant::now());
    }
    pub fn int_pressed(&mut self) {
        self.last_edge_int_press = Some(Instant::now());
    }
    pub fn continue_pressed(&mut self) {
        self.last_continue_press = Some(Instant::now());
    }
    pub fn set_continue_possible(&mut self, possible: bool) {
        self.is_continue_possible = possible;
    }
    pub fn set_edge_int_possible(&mut self, possible: bool) {
        self.is_edge_int_possible = possible;
    }
    pub fn set_autorun_on(&mut self, on: bool) {
        self.is_autorun_on = on;
    }
    pub fn set_asm_step_on(&mut self, on: bool) {
        self.is_asm_step_on = on;
    }
}

fn is_within_highlight_dur(instant: &mut Option<Instant>) -> bool {
    match *instant {
        Some(instant) if instant.elapsed() <= HIGHLIGHT_DURATION => true,
        Some(_) | None => false,
    }
}
