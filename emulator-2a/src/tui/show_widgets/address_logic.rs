use emulator_2a_lib::machine::Machine;
use lazy_static::lazy_static;

use std::{collections::HashMap, marker::PhantomData};

use tui::{buffer::Buffer, layout::Rect, style::Style, widgets::StatefulWidget};

use crate::{helpers::YELLOW, tui::wrapper_widgets::MinimumSize};

/// Base string for the widget.
const WIDGET_BASE: &'static str = r#"
Address Logic:                         ┏
                      Instruction Reg╶┄┨◌◌◌◌ A8..A5
                                   NA4╶┨◌ A4
      OP10╶┐                       NA3╶┨◌ A3
      ┏AM2┓│          ┏AM1┓        NA2╶┨◌ A2
    1╶┨   ┃│        0╶┨   ┃      ┏AM4┓ ┃
   CF╶┨◌  ┃│┏AL3┓   1╶┨   ┃  NA1╶┨◌  ┠─┨◌ A1
   ZF╶┨◌  ┃└┨◌  ┠─────┨   ┃ OP11╶┨◌  ┃ ┃
   NF╶┨◌  ┠─┨ =1┃  CF╶┨◌  ┃      ┗━┯━┛ ┃
      ┗┯┯━┛ ┗━━━┛  CO╶┨◌  ┃   MAC2╶┘   ┃
  OP01╶┘│          ZO╶┨◌  ┃      ┏AM3┓ ┃
   OP00╶┘   ┏AL2┓  NO╶┨◌  ┠──────┨   ┠─┨◌ A0
        IEF╶┨◌  ┠─────┨◌  ┃ OP10╶┨◌  ┃ ┗
      ┏AL1┓┌┨  &┃     ┗┯┯┯┛      ┗━┯━┛
LINT╶┄┨◌  ┠┘┗━━━┛ MAC1╶┘││    MAC2╶┘
IFF1╶┄┨◌≥1┃        MAC0╶┘│
      ┗━━━┛          NA0╶┘
      "#;
const MINIMUM_ALLOWED_WIDTH: u16 = 50;
const MINIMUM_ALLOWED_HEIGHT: u16 = 16;

/// Relative locations in the [`WIDGET_BASE`] with names.
const DOT_POSITIONS: [(&'static str, u16, u16); 24] = [
    ("a8", 40, 1),
    ("a7", 41, 1),
    ("a6", 42, 1),
    ("a5", 43, 1),
    ("a4", 40, 2),
    ("a3", 40, 3),
    ("a2", 40, 4),
    ("a1", 40, 6),
    ("a0", 40, 11),
    ("am1-cf", 23, 8),
    ("am1-co", 23, 9),
    ("am1-zo", 23, 10),
    ("am1-no", 23, 11),
    ("am1-al2", 23, 12),
    ("am2-cf", 7, 6),
    ("am2-zf", 7, 7),
    ("am2-nf", 7, 8),
    ("am3-op10", 34, 12),
    ("am4-na1", 34, 6),
    ("am4-op11", 34, 7),
    ("al1-lint", 7, 14),
    ("al1-iff1", 7, 15),
    ("al2-ief", 13, 12),
    ("al3-op10", 13, 7),
];

lazy_static! {
    static ref DOT_POSITION_MAP: HashMap<&'static str, (u16, u16)> = DOT_POSITIONS
        .iter()
        .map(|(id, x, y)| (*id, (*x, *y)))
        .collect();
    static ref WIDGET_BASE_LINES: Vec<&'static str> = WIDGET_BASE
        .lines()
        .map(str::trim_end)
        .filter(|s| !s.is_empty())
        .collect();
}

/// ```text
/// Address Logic:                          ┏
///                        Instruction Reg╶┄┨●●●● A8..A5
///                                     NA4╶┨● A4
///        OP10╶┐                       NA3╶┨● A3
///        ┏AM2┓│          ┏AM1┓        NA2╶┨● A2
///      1╶┨   ┃│        0╶┨   ┃      ┏AM4┓ ┃
///     CF╶┨●  ┃│┏AL3┓   1╶┨   ┃  NA1╶┨●  ┠─┨● A1
///     ZF╶┨●  ┃└┨●  ┠─────┨   ┃ OP11╶┨●  ┃ ┃
///     NF╶┨●  ┠─┨ =1┃  CF╶┨●  ┃      ┗━┯━┛ ┃
///        ┗┯┯━┛ ┗━━━┛  CO╶┨●  ┃   MAC2╶┘   ┃
///    OP01╶┘│          ZO╶┨●  ┃      ┏AM3┓ ┃
///     OP00╶┘  ┏AL2┓   NO╶┨●  ┠──────┨   ┠─┨● A0
///         IEF╶┨●  ┠──────┨●  ┃ OP10╶┨●  ┃ ┗
///       ┏AL1┓┌┨  &┃      ┗┯┯┯┛      ┗━┯━┛
/// LINT╶┄┨○  ┠┘┗━━━┛  MAC1╶┘││    MAC2╶┘
/// IFF1╶┄┨●≥1┃         MAC0╶┘│
///       ┗━━━┛           NA0╶┘
/// ```
pub struct AddressLogicWidget;

impl StatefulWidget for AddressLogicWidget {
    type State = Machine;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        for (nr, line) in WIDGET_BASE_LINES.iter().enumerate() {
            buf.set_string(area.x, area.y + nr as u16, line, Style::default())
        }
        let signals = state.signals();
        set_bit(buf, area, "a8", signals.a8());
        set_bit(buf, area, "a7", signals.a7());
        set_bit(buf, area, "a6", signals.a6());
        set_bit(buf, area, "a5", signals.a5());
        set_bit(buf, area, "a4", signals.na4());
        set_bit(buf, area, "a3", signals.na3());
        set_bit(buf, area, "a2", signals.na2());
        set_bit(buf, area, "a1", signals.na1());
        set_bit(buf, area, "a0", signals.na0());
        set_bit(buf, area, "am1-cf", signals.carry_flag());
        set_bit(buf, area, "am1-co", signals.carry_out());
        set_bit(buf, area, "am1-zo", signals.zero_out());
        set_bit(buf, area, "am1-no", signals.negative_out());
        set_bit(buf, area, "am1-al2", signals.address_logic_2());
        set_bit(buf, area, "am2-cf", signals.carry_flag());
        set_bit(buf, area, "am2-zf", signals.zero_flag());
        set_bit(buf, area, "am2-nf", signals.negative_flag());
        set_bit(buf, area, "am3-op10", signals.op10());
        set_bit(buf, area, "am4-na1", signals.na1());
        set_bit(buf, area, "am4-op11", signals.op11());
        set_bit(buf, area, "al1-lint", signals.level_interrupt());
        set_bit(buf, area, "al1-iff1", signals.interrupt_flipflop_1());
        set_bit(buf, area, "al2-ief", signals.interrupt_enable_flag());
        set_bit(buf, area, "al3-op10", signals.op10());
    }
}

/// Helper to get the absolute position for the given `key`.
fn get_pos(area: Rect, key: &'static str) -> (u16, u16) {
    if let Some(pos) = DOT_POSITION_MAP.get(key) {
        (area.x + pos.0, area.y + pos.1)
    } else {
        panic!("BUG: No such key: {}", key);
    }
}

/// Helper to set bit 'dots' in the buffer.
fn set_bit(buf: &mut Buffer, area: Rect, key: &'static str, value: bool) {
    let pos = get_pos(area, key);
    let (bit_char, style) = if value {
        ("●", *YELLOW)
    } else {
        ("○", Style::default())
    };
    buf.set_string(pos.0, pos.1, bit_char, style);
}
