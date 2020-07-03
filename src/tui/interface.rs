//! The basis of the TUI user interface.
//! ```text
//! ┌─┤ Minirechner 2a ├────────────────────┐━╸Info╺━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//! │                                       │Program:         12-halt-on-int.asm
//! │ Outputs:                              │Frequency:                  7.41MHz
//! │ 00000000 00000000                     │Measured Frequency:          0.00Hz
//! │       FF       FE                     │State:                      Running
//! │                                       │━╸Program╺━━━━━━━━━━━━━━━━━━━━━━━━━
//! │ Inputs:                               │     .ORG 0                    ; Pr
//! │ 00000000 00000000 00000000 00000000   │>    JR MAIN                   ; Sp
//! │       FF       FE       FD       FC   │     JR INTERRUPT              ; Di
//! │                                       │ MAIN:
//! │ Registers:                            │     EI                        ; Er
//! │ R0 00000000                           │     BITS (0xF9), 0x01         ; Se
//! │ R1 00000000                           │     LDSP 0xEF                 ; De
//! │ R2 00000000                           │ LOOP:                         ; En
//! │ PC 00000000                           │     JR LOOP
//! │ FR 00000000                           │ INTERRUPT:
//! │ SP 00000000                           │━╸Keybindings╺━━━━━━━━━━━━━━━━━━━━━
//! │ R6 00000000                           │Clock                         Enter
//! │ R7 00000000                           │Toggle autorun               CTRL+A
//! │                                       │Toggle asm step              CTRL+W
//! │                                       │Reset                        CTRL+R
//! │                                       │Edge interrupt               CTRL+E
//! │                                       │Continue                     CTRL+L
//! │                                       │━╸Commands╺━━━━━━━━━━━━━━━━━━━━━━━━
//! │                                       │load PATH          Load asm program
//! │                                   ... │set …             Change a settings
//! │                                       │unset …        Unset a bool setting
//! │───────────────────────────────────────│show …       Select part to display
//! │> █                                    │quit               Exit the program
//! └───────────────────────────────────────┘───────────────────────────────────
//! ```

use lazy_static::lazy_static;
use tui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
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

pub struct Interface {
    /// For updating some things not every frame.
    counter: usize,
}

impl Interface {
    /// Initialize a new interface.
    pub fn new() -> Self {
        let counter = 0;
        Interface { counter }
    }
    /// Draw the interface using information from the given [`Tui`]
    pub fn draw<'b>(&mut self, tui: &'b mut Tui, f: &mut Frame<Backend>) {
        // Increment draw counter
        self.counter = self.counter.overflowing_add(1).0;
        let area = f.size();
        let area = Rect::new(area.x, area.y, area.width, area.height);
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
    }
}
