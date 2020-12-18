//! Everything related to drawing the [`MemoryWidget`].
use tui::{buffer::Buffer, layout::Rect, style::Style, widgets::Widget};

use crate::helpers;

const MINIMUM_ALLOWED_WIDTH_FOR_MEMORY_DISPLAY: u16 = 50;
const MINIMUM_ALLOWED_HEIGHT_FOR_MEMORY_DISPLAY: u16 = 17;

/// A widget for displaying the memory.
///
/// The first parameter is a reference to the memory.
///
/// # Example
///
/// ```text
/// Memory:
///    _0 _1 _2 _3 _4 _5 _6 _7 _8 _9 _A _B _C _D _E _F
/// 0_ 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
/// 1_ 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
/// 2_ 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
/// 3_ 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
/// 4_ 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
/// 5_ 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
/// 6_ 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
/// 7_ 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
/// 8_ 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
/// 9_ 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
/// A_ 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
/// B_ 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
/// C_ 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
/// D_ 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
/// E_ 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
/// ```
pub struct MemoryWidget<'a>(pub &'a [u8; 0xF0]);

impl Widget for MemoryWidget<'_> {
    fn render(self, mut area: Rect, buf: &mut Buffer) {
        // Display title
        buf.set_string(area.left(), area.top(), "Memory:", *helpers::DIMMED);
        area.y += 1;
        area.height -= 1;
        // Make sure, that we have enough space!
        if area.width < MINIMUM_ALLOWED_WIDTH_FOR_MEMORY_DISPLAY {
            buf.set_string(
                area.left(),
                area.top() + 1,
                "Display width too small!",
                *helpers::RED_BOLD,
            );
        } else if area.height < MINIMUM_ALLOWED_HEIGHT_FOR_MEMORY_DISPLAY {
            buf.set_string(
                area.left(),
                area.top() + 1,
                "Display height too small!",
                *helpers::RED_BOLD,
            );
        } else {
            for hex in 0..0x10_u8 {
                // Top row of annotations
                let area_x = area.left() + 3 + hex as u16 * 3;
                buf.set_string(area_x, area.top(), format!("_{:X}", hex), *helpers::DIMMED);
            }
            for hex in 0..0xF_u8 {
                // Left row of annotations
                let area_y = area.top() + 1 + hex as u16;
                buf.set_string(area.left(), area_y, format!("{:X}_", hex), *helpers::DIMMED);
            }
            area.x += 3;
            area.y += 1;
            area.width -= 3;
            area.height -= 1;
            // Iterate over the memory
            for (index, content) in self.0.iter().enumerate() {
                // Draw non-empty cells bold
                let style = if *content == 0 {
                    Style::default()
                } else {
                    *helpers::BOLD
                };
                let cell = hex_str(content);
                let x_pos = area.left() + (index as u16 % 0x10) * 3;
                let y_pos = area.top() + index as u16 / 0x10;
                buf.set_string(x_pos, y_pos, &cell, style)
            }
        }
    }
}

/// Format a hexadecimal right padded
fn hex_str(hex: &u8) -> String {
    format!("{:>02X}", hex)
}
