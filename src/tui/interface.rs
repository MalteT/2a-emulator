//! The basis of the TUI user interface.

use lazy_static::lazy_static;
use tui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, StatefulWidget, Text, Widget},
};

use crate::{
    helpers,
    tui::{input::InputWidget, ProgramHelpSidebar, SupervisorWrapper, Tui},
};

pub const MINIMUM_ALLOWED_WIDTH: u16 = 76;
pub const MINIMUM_ALLOWED_HEIGHT: u16 = 28;
const RIGHT_SIDEBAR_WIDTH: u16 = 35;
const INPUT_AREA_HEIGHT: u16 = 2;

const ERROR_HEIGHT_TOO_SMALL: &[TextSlice] = &[
    TextSlice::Raw("Terminal height too small!\n Please"),
    TextSlice::Colored(" resize your terminal", Color::Yellow),
    TextSlice::Raw(" or"),
    TextSlice::Colored(" decrease your font size", Color::Yellow),
    TextSlice::Raw("!"),
];
const ERROR_WIDTH_TOO_SMALL: &[TextSlice] = &[
    TextSlice::Raw("Terminal width too small!\n Please"),
    TextSlice::Colored(" resize your terminal", Color::Yellow),
    TextSlice::Raw(" or"),
    TextSlice::Colored(" decrease your font size", Color::Yellow),
    TextSlice::Raw("!"),
];

lazy_static! {
    static ref BLK_ERROR: Block<'static> = Block::default()
        .title("─┤ Error ├")
        .title_style(*helpers::RED_BOLD)
        .borders(Borders::ALL)
        .border_style(*helpers::RED_BOLD);
}

/// The main view shown on the left of the TUI.
///
/// # Example
///
/// ```text
/// ┌─┤ Minirechner 2a ├────────────────────┐
/// │                                       │
/// │ Outputs:                              │
/// │ 00000000 00000000                     │
/// │       FF       FE                     │
/// │                                       │
/// │ Inputs:                               │
/// │ 00000000 00000000 00000000 00000000   │
/// │       FF       FE       FD       FC   │
/// │                                       │
/// │ Registers:                            │
/// │ R0 00000000                           │
/// │ R1 00000000                           │
/// │ R2 00000000                           │
/// │ PC 00000000                           │
/// │ FR 00000000                           │
/// │ SP 00000000                           │
/// │ R6 00000000                           │
/// │ R7 00000000                           │
/// │                                       │
/// │                                       │
/// │                                       │
/// │                                       │
/// │                                       │
/// │                                       │
/// │                                   ... │
/// │                                       │
/// │───────────────────────────────────────│
/// │> █                                    │
/// └───────────────────────────────────────┘
/// ```
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
        InputWidget.render(input_field_area, buf, &mut state.input_field);
        // Render the rest of the main view, registers and the shown part.
        let main_machine_area = Rect {
            height: area_inside_block.height - INPUT_AREA_HEIGHT,
            ..area_inside_block
        };
        SupervisorWrapper::new().render(main_machine_area, buf, &mut state.supervisor);
    }
}

/// The main widget for displaying the TUI.
///
/// # Example
///
/// ```text
/// ┌─┤ Minirechner 2a ├────────────────────┐━╸Info╺━━━━━━━━━━━━━━━━━━━━━━━━━━━━
/// │                                       │Program:         12-halt-on-int.asm
/// │ Outputs:                              │Frequency:                  7.41MHz
/// │ 00000000 00000000                     │Measured Frequency:          0.00Hz
/// │       FF       FE                     │State:                      Running
/// │                                       │━╸Program╺━━━━━━━━━━━━━━━━━━━━━━━━━
/// │ Inputs:                               │     .ORG 0                    ; Pr
/// │ 00000000 00000000 00000000 00000000   │>    JR MAIN                   ; Sp
/// │       FF       FE       FD       FC   │     JR INTERRUPT              ; Di
/// │                                       │ MAIN:
/// │ Registers:                            │     EI                        ; Er
/// │ R0 00000000                           │     BITS (0xF9), 0x01         ; Se
/// │ R1 00000000                           │     LDSP 0xEF                 ; De
/// │ R2 00000000                           │ LOOP:                         ; En
/// │ PC 00000000                           │     JR LOOP
/// │ FR 00000000                           │ INTERRUPT:
/// │ SP 00000000                           │━╸Keybindings╺━━━━━━━━━━━━━━━━━━━━━
/// │ R6 00000000                           │Clock                         Enter
/// │ R7 00000000                           │Toggle autorun               CTRL+A
/// │                                       │Toggle asm step              CTRL+W
/// │                                       │Reset                        CTRL+R
/// │                                       │Edge interrupt               CTRL+E
/// │                                       │Continue                     CTRL+L
/// │                                       │━╸Commands╺━━━━━━━━━━━━━━━━━━━━━━━━
/// │                                       │load PATH          Load asm program
/// │                                   ... │set …             Change a settings
/// │                                       │unset …        Unset a bool setting
/// │───────────────────────────────────────│show …       Select part to display
/// │> █                                    │quit               Exit the program
/// └───────────────────────────────────────┘───────────────────────────────────
/// ```
pub struct Interface;

impl StatefulWidget for Interface {
    type State = Tui;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if area.width < MINIMUM_ALLOWED_WIDTH {
            ErrorWidget(ERROR_WIDTH_TOO_SMALL).render(area, buf)
        } else if area.height < MINIMUM_ALLOWED_HEIGHT {
            ErrorWidget(ERROR_HEIGHT_TOO_SMALL).render(area, buf)
        } else {
            // This is the area for the main component, the [`MainView`].
            let main_view_area = Rect {
                width: area.width - RIGHT_SIDEBAR_WIDTH,
                ..area
            };
            MainView.render(main_view_area, buf, state);
            // This is the area for the sidebar on the right, the [`ProgramHelpSidebar`].
            let help_area = Rect {
                x: area.right() - RIGHT_SIDEBAR_WIDTH,
                width: RIGHT_SIDEBAR_WIDTH,
                ..area
            };
            ProgramHelpSidebar.render(help_area, buf, state);
        }
    }
}

/// Displays a rect with information about the error.
pub struct ErrorWidget<'a, 'b>(pub &'a [TextSlice<'b>]);

impl Widget for ErrorWidget<'_, '_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let text: Vec<_> = self
            .0
            .iter()
            .map(|text| match text {
                TextSlice::Raw(text) => Text::raw(*text),
                TextSlice::Colored(text, color) => {
                    Text::styled(*text, Style::default().fg(*color).modifier(Modifier::BOLD))
                }
            })
            .collect();
        Paragraph::new(text.iter())
            .alignment(Alignment::Center)
            .block(*BLK_ERROR)
            .render(area, buf)
    }
}

/// An enum represanting a str that may have some color attached.
pub enum TextSlice<'a> {
    /// Unstyled text.
    Raw(&'a str),
    /// Text with associated color.
    Colored(&'a str, Color),
}
