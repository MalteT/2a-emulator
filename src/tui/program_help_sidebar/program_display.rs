//! Everythin related to drawing the [`ProgramDisplayWidget`].
use parser2a::asm::Line;
use tui::{buffer::Buffer, layout::Rect, style::Style, widgets::StatefulWidget};

use std::ops::Range;

use crate::{compiler::ByteCode, helpers};

/// Maximum number of lines shown before and after the current line.
const MAX_LINES_OF_CONTEXT: usize = 3;

/// This Widget can render the current program.
/// The first argument is the PC (program counter) value.
///
/// # Example
///
/// ```text
/// ━╸Program╺━━━━━━━━━━━━━━━━━━━━━━━━━
///      CLR R0
///      CLR R1
///  LOOP:
///      LD R0, (0xFC)
///      LD R1, (0xFD)
///      ADD R0, R1
/// >    ST (0xFF), R0
///      JR LOOP
/// ```
pub struct ProgramDisplayWidget(pub u8);

impl StatefulWidget for ProgramDisplayWidget {
    type State = ProgramDisplayState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let header = super::make_header("Program", area.width);
        buf.set_string(area.left(), area.top(), header, *helpers::DIMMED_BOLD);
        let area = Rect {
            y: area.y + 1,
            height: area.height - 1,
            ..area
        };
        let area_height = area.height as usize;
        let current_line_idx = state.index_for_address(self.0).unwrap_or_default();
        let lines_of_context = MAX_LINES_OF_CONTEXT.min(area_height / 2);
        let first_context_line_idx = current_line_idx.saturating_sub(lines_of_context);
        let last_context_line_idx = current_line_idx.saturating_add(lines_of_context);
        let last_displayed_line_idx = state.current_top_line_idx + area_height;
        // Update the `current_top_line_idx` in case not all
        // current context can be displayed
        // If the last context line is not displayed, shift the view
        // upwards by subtracting the diff from the `current_top_line_idx`.
        if last_displayed_line_idx < last_context_line_idx {
            let diff = last_context_line_idx - last_displayed_line_idx;
            state.current_top_line_idx = state.current_top_line_idx + diff;
        }
        state.current_top_line_idx = state
            .current_top_line_idx
            // Never start after the first line of the current context
            .min(first_context_line_idx)
            // Never start so far down, that we have an empty space after the
            // last line of the program
            .min(state.lines.len().saturating_sub(area_height));
        // Iterate over lines from `current_top_line_idx` and stop after
        // we have enough to fill the area height.
        for (idx, (_range, line)) in state
            .lines
            .iter()
            .enumerate()
            .skip(state.current_top_line_idx)
            .take(area_height)
        {
            // Subtract the skipped lines for correct positioning
            let offset = idx.saturating_sub(state.current_top_line_idx) as u16;
            if idx == current_line_idx {
                // Draw the highlighter for the current line
                buf.set_string(area.left(), area.top() + offset, ">", *helpers::YELLOW_BOLD);
            }
            buf.set_stringn(
                area.left() + 1,
                area.top() + offset,
                line,
                area.width as usize,
                if idx == current_line_idx {
                    *helpers::BOLD
                } else {
                    Style::default()
                },
            );
        }
    }
}

/// State needed to display the [`ProgramDisplayWidget`].
pub struct ProgramDisplayState {
    /// The lines of the program with corresponding ranges of
    /// program counter values.
    pub lines: Vec<(Range<u8>, String)>,
    /// The index of the topmost line currently displayed.
    pub current_top_line_idx: usize,
}

impl ProgramDisplayState {
    /// Create an empty default state.
    pub fn empty() -> Self {
        ProgramDisplayState {
            lines: vec![],
            current_top_line_idx: 0,
        }
    }
    /// Create the state from reading [`ByteCode`] input.
    pub fn from_bytecode(bytecode: &ByteCode) -> Self {
        let mut byte_counter: u8 = 0;
        let lines = bytecode
            .lines
            .iter()
            .filter(|(line, _)| *line != Line::Empty(None))
            .map(|(line, bytes)| {
                let string = line.to_string();
                let from = byte_counter;
                let to = byte_counter + bytes.len() as u8;
                byte_counter = to;
                (from..to, string)
            })
            .collect();
        ProgramDisplayState {
            lines,
            current_top_line_idx: 0,
        }
    }
    /// Get the program line that is contained at `addr` in memory.
    ///
    /// The returned index refers to the program line that is
    /// executed if `addr` refers to the value of the program counter.
    fn index_for_address(&self, addr: u8) -> Option<usize> {
        self.lines
            .iter()
            // Keep track of the index
            .enumerate()
            // Only keep entries where the range contains our address
            // This _should_ leave us with at most a single element
            .filter(|(_, (range, _))| range.contains(&addr))
            // Discard everything but the index
            .map(|(idx, _)| idx)
            // Return the first index
            .next()
    }
}
