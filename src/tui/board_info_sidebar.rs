use tui::{buffer::Buffer, layout::Rect, style::Style, widgets::StatefulWidget};

use crate::{
    helpers,
    machine::{Board, DASR},
    tui::SupervisorWrapperState,
};

const FAN_RPM_OFFSET: u16 = 0;
const IRG_OFFSET: u16 = 2;
const ORG1_OFFSET: u16 = 3;
const ORG2_OFFSET: u16 = 4;
const TEMP_OFFSET: u16 = 6;
const AI1_OFFSET: u16 = 7;
const AI2_OFFSET: u16 = 8;
const AO1_OFFSET: u16 = 9;
const AO2_OFFSET: u16 = 10;
const UIO1_OFFSET: u16 = 12;
const UIO2_OFFSET: u16 = 13;
const UIO3_OFFSET: u16 = 14;
const J1_OFFSET: u16 = 16;
const J2_OFFSET: u16 = 17;

/// Renders additional information about the MR2DA2 extension board.
///
/// # Example
///
/// ```text
///   16 RPM +
///
///  0x0A  IRG
///  0x01 ORG1
///  0x02 ORG2
///
/// 3.20V TEMP
/// 1.00V  AI1
/// 5.00V  AI2
/// 0.01V  AO1
/// 0.02V  AO2
///
///   » 1 UIO1
///   » 1 UIO2
///   » 1 UIO3
///
///     ╼━╾ J1
///     ╼━╾ J2
/// ```
pub struct BoardInfoSidebarWidget;

impl StatefulWidget for BoardInfoSidebarWidget {
    /// Input registers FC, FD, FE, FF.
    type State = SupervisorWrapperState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_fan_rpm(area, buf, state);

        let board = state.machine().bus.board();
        render_digital_io(area, buf, board);
        render_analog_io(area, buf, board);
        render_uios(area, buf, board);
        render_jumper(area, buf, board);
    }
}

/// Render the current fan speed in rpm.
///
/// # Example
///
/// ```text
/// 16 RPM +
/// ```
pub fn render_fan_rpm(area: Rect, buf: &mut Buffer, state: &mut SupervisorWrapperState) {
    // Display fan speed in rpm
    if *state.machine().bus.board().fan_rpm() != 0 {
        let s = format!(
            "{:>4} RPM {}",
            state.machine().bus.board().fan_rpm(),
            if state.draw_counter % 10 < 5 {
                "×"
            } else {
                "+"
            }
        );
        buf.set_string(
            area.right() - 10,
            area.y + FAN_RPM_OFFSET,
            s,
            Style::default(),
        );
    }
}

/// Render the digital I/O pin's states.
///
/// # Example
///
/// ```text
/// 0x0A  IRG
/// 0x01 ORG1
/// 0x02 ORG2
/// ```
pub fn render_digital_io(area: Rect, buf: &mut Buffer, board: &Board) {
    if *board.irg() != 0 {
        let s = format!("{:>02X}  IRG", board.irg());
        buf.set_string(
            area.right() - 9,
            area.y + IRG_OFFSET,
            "0x",
            *helpers::DIMMED,
        );
        buf.set_string(area.right() - 7, area.y + IRG_OFFSET, s, Style::default());
    }
    if *board.org1() != 0 {
        let s = format!("{:>02X} ORG1", board.org1());
        buf.set_string(
            area.right() - 9,
            area.y + ORG1_OFFSET,
            "0x",
            *helpers::DIMMED,
        );
        buf.set_string(area.right() - 7, area.y + ORG1_OFFSET, s, Style::default());
    }
    if *board.org2() != 0 {
        let s = format!("{:>02X} ORG2", board.org2());
        buf.set_string(
            area.right() - 9,
            area.y + ORG2_OFFSET,
            "0x",
            *helpers::DIMMED,
        );
        buf.set_string(area.right() - 7, area.y + ORG2_OFFSET, s, Style::default());
    }
}

/// Render the analog I/O pin's states.
///
/// # Example
///
/// ```text
/// 3.20V TEMP
/// 1.00V  AI1
/// 5.00V  AI2
/// 0.01V  AO1
/// 0.02V  AO2
/// ```
pub fn render_analog_io(area: Rect, buf: &mut Buffer, board: &Board) {
    if *board.temp() != 0.0 {
        let s = format!("{:.2}V TEMP", board.temp());
        buf.set_string(area.right() - 10, area.y + TEMP_OFFSET, s, Style::default());
    }
    if board.analog_inputs()[0] != 0.0 {
        let s = format!("{:.2}V  AI1", board.analog_inputs()[0]);
        buf.set_string(area.right() - 10, area.y + AI1_OFFSET, s, Style::default());
    }
    if board.analog_inputs()[1] != 0.0 {
        let s = format!("{:.2}V  AI2", board.analog_inputs()[1]);
        buf.set_string(area.right() - 10, area.y + AI2_OFFSET, s, Style::default());
    }
    if board.analog_outputs()[0] != 0.0 {
        let s = format!("{:.2}V  AO1", board.analog_outputs()[0]);
        buf.set_string(area.right() - 10, area.y + AO1_OFFSET, s, Style::default());
    }
    if board.analog_outputs()[1] != 0.0 {
        let s = format!("{:.2}V  AO2", board.analog_outputs()[1]);
        buf.set_string(area.right() - 10, area.y + AO2_OFFSET, s, Style::default());
    }
}

/// Render the states of the Universal I/O pins.
///
/// # Example
/// ```text
/// » 1 UIO1
/// » 1 UIO2
/// » 1 UIO3
/// ```
pub fn render_uios(area: Rect, buf: &mut Buffer, board: &Board) {
    let uio1 = board.dasr().contains(DASR::UIO_1);
    let uio2 = board.dasr().contains(DASR::UIO_2);
    let uio3 = board.dasr().contains(DASR::UIO_3);
    if board.uio_dir()[0] && uio1 {
        let s = format!("« {} UIO1", uio1 as u8);
        buf.set_string(area.right() - 8, area.y + UIO1_OFFSET, s, Style::default());
    } else if uio1 {
        let s = format!("» {} UIO1", uio1 as u8);
        buf.set_string(area.right() - 8, area.y + UIO1_OFFSET, s, Style::default());
    }
    if board.uio_dir()[1] && uio2 {
        let s = format!("« {} UIO2", uio2 as u8);
        buf.set_string(area.right() - 8, area.y + UIO2_OFFSET, s, Style::default());
    } else if uio2 {
        let s = format!("» {} UIO2", uio2 as u8);
        buf.set_string(area.right() - 8, area.y + UIO2_OFFSET, s, Style::default());
    }
    if board.uio_dir()[2] && uio3 {
        let s = format!("« {} UIO3", uio3 as u8);
        buf.set_string(area.right() - 8, area.y + UIO3_OFFSET, s, Style::default());
    } else if uio3 {
        let s = format!("» {} UIO3", uio3 as u8);
        buf.set_string(area.right() - 8, area.y + UIO3_OFFSET, s, Style::default());
    }
}

/// Render the states of the jumpers.
///
/// # Example
///
/// ```text
/// ╼━╾ J1
/// ╼━╾ J2
/// ```
pub fn render_jumper(area: Rect, buf: &mut Buffer, board: &Board) {
    if board.dasr().contains(DASR::J1) {
        buf.set_string(
            area.right() - 6,
            area.y + J1_OFFSET,
            "╼━╾ J1",
            Style::default(),
        );
    }
    if board.dasr().contains(DASR::J2) {
        buf.set_string(
            area.right() - 6,
            area.y + J2_OFFSET,
            "╼━╾ J2",
            Style::default(),
        );
    }
}
