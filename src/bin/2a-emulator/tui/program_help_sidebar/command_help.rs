//! Everything related to drawing the [`CommandHelpWidget`].
use tui::{buffer::Buffer, layout::Rect, widgets::Widget};

use super::{SpacedStr, HEADER_HEIGHT};
use crate::helpers;

const COMMAND_HELP_DEFAULT: &[(&str, &str)] = &[
    ("load PATH", "Load asm program"),
    ("set …", "Change a settings"),
    ("unset …", "Unset a bool setting"),
    ("show …", "Select part to display"),
    ("quit", "Exit the program"),
];
const COMMAND_HELP_SET: &[(&str, &str)] = &[
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
const COMMAND_HELP_UNSET: &[(&str, &str)] = &[
    ("J1", "MR2DA2 jumper 1"),
    ("J2", "MR2DA2 jumper 2"),
    ("UIO1", "MR2DA2 universal IO1"),
    ("UIO2", "MR2DA2 universal IO2"),
    ("UIO3", "MR2DA2 universal IO3"),
];
const COMMAND_HELP_SHOW: &[(&str, &str)] = &[
    ("memory", "Show the main memory"),
    ("register", "Show the registers"),
];
const COMMAND_HELP_LOAD: &[(&str, &str)] = &[("PATH", "Path to the program")];

/// Help widget that shows input completions.
///
/// # Example
///
/// ```text
/// ━╸Commands╺━━━━━━━━━━━━━━━━━━━━━━━━
/// load PATH         Load asm program
/// set …            Change a settings
/// unset …       Unset a bool setting
/// show …      Select part to display
/// quit              Exit the program
/// ```
pub struct CommandHelpWidget<'a>(pub &'a str);

impl<'a> CommandHelpWidget<'a> {
    // Calculate the height needed to draw the command help based on the
    // current user input.
    pub fn calculate_height(input: &str) -> u16 {
        let input = input.to_lowercase();
        let line_count = if input.starts_with("load ") {
            COMMAND_HELP_LOAD.len()
        } else if input.starts_with("set ") {
            COMMAND_HELP_SET.len()
        } else if input.starts_with("unset ") {
            COMMAND_HELP_UNSET.len()
        } else if input.starts_with("show ") {
            COMMAND_HELP_SHOW.len()
        } else {
            COMMAND_HELP_DEFAULT.len()
        };
        // Number of help lines + one for the header
        line_count as u16 + HEADER_HEIGHT
    }
}

impl<'a> Widget for CommandHelpWidget<'a> {
    fn render(self, mut area: Rect, buf: &mut Buffer) {
        // Current input in lowercase
        let input = self.0.to_lowercase();
        // Render header
        let header = super::make_header("Commands", area.width);
        buf.set_string(
            area.left(),
            area.top(),
            header,
            if input.is_empty() {
                *helpers::DIMMED_BOLD
            } else {
                *helpers::YELLOW_BOLD
            },
        );
        // Render actual help
        let help_array = if input.starts_with("load ") {
            // TODO: Complete paths
            COMMAND_HELP_LOAD
        } else if input.starts_with("set ") {
            COMMAND_HELP_SET
        } else if input.starts_with("unset ") {
            COMMAND_HELP_UNSET
        } else if input.starts_with("show ") {
            COMMAND_HELP_SHOW
        } else {
            COMMAND_HELP_DEFAULT
        };

        for (left, right) in help_array {
            area.y += 1;
            area.height -= 1;
            let mut spaced = SpacedStr::from(left, right);
            if !input.is_empty() {
                spaced = spaced
                    .left_style(&helpers::BOLD)
                    .right_style(&helpers::BOLD);
            }
            spaced.render(area, buf);
        }
    }
}
