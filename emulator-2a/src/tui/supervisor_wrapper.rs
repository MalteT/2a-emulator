use emulator_2a_lib::{
    compiler::ByteCode,
    machine::{Machine, StepMode},
};
use tui::{
    buffer::Buffer,
    layout::{Margin, Rect},
    style::Style,
    widgets::{StatefulWidget, Widget},
};

use std::{
    ops::{Deref, DerefMut},
    path::PathBuf,
    time::{Duration, Instant},
};

use crate::{
    args::InitialMachineConfiguration,
    helpers,
    tui::{
        display::Display,
        show_widgets::{MemoryWidget, RegisterBlockWidget},
        BoardInfoSidebarWidget,
    },
};

const ONE_SPACE: u16 = 1;
const BYTE_WIDTH: u16 = 8;
const OUTPUT_REGISTER_WIDGET_WIDTH: u16 = 2 * BYTE_WIDTH + ONE_SPACE;
const OUTPUT_REGISTER_WIDGET_HEIGHT: u16 = 3;
const INPUT_REGISTER_WIDGET_WIDTH: u16 = 4 * BYTE_WIDTH + 3 * ONE_SPACE;
const INPUT_REGISTER_WIDGET_HEIGHT: u16 = 3;
const BOARD_INFO_SIDEBAR_WIDGET_WIDTH: u16 = 20;
const SHOW_PART_START_Y_OFFSET: u16 =
    INPUT_REGISTER_WIDGET_HEIGHT + OUTPUT_REGISTER_WIDGET_HEIGHT + 2 * ONE_SPACE;

const NUMBER_OF_MEASUREMENTS: usize = 30;
const DEFAULT_CLK_FREQUENCY: f32 = 7.3728e6;

/// Widget for drawing the machine.
///
/// # Example
///
/// ```text
/// Outputs:
/// 00001011 00000000
///       FF       FE
///
/// Inputs:
/// 00000000 00000000 00001010 00000001
///       FF       FE       FD       FC
///
/// Registers:
/// R0 00000001
/// R1 00001010
/// R2 00000000
/// PC 00000101
/// FR 00000000
/// SP 00000000
/// R6 00000001
/// R7 11111100
/// ```
pub struct MachineWidget;

/// State necessary to draw the [`MachineWidget`] widget.
pub struct MachineState {
    /// The machine that is drawn.
    pub machine: Machine,
    /// The part currently displayed by the TUI.
    pub part: Part,
    /// Counting draw cycles.
    pub draw_counter: usize,
    /// Is the auto run mode active?
    pub auto_run_mode: bool,
    /// Currenly active program.
    program: Option<PathBuf>,
    /// Frequency measurement utility.
    freq_measurements: FreqMeasurements,
}

/// Helper struct for frequency measurements.
struct FreqMeasurements {
    /// The index of the oldest measurement.
    oldest_index: usize,
    /// The measurements.
    measurements: [f32; NUMBER_OF_MEASUREMENTS],
    /// Last time a clock occured.
    last_clk: Instant,
}

/// Displayable parts.
///
/// These parts have a widget implementation and can be rendered by the TUI.
/// Selecting these parts can be done using the `show ...` command interactively.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Part {
    RegisterBlock,
    Memory,
}

impl MachineState {
    /// Create a new MachineState.
    ///
    /// The given [`InitialMachineConfiguration`] is used to configure the underlying
    /// [`Machine`]. Initially the additional displayed part is the [`Part::RegisterBlock`].
    pub fn new(conf: &InitialMachineConfiguration) -> Self {
        MachineState {
            part: Part::RegisterBlock,
            machine: Machine::new(conf.clone().into()),
            draw_counter: 0,
            auto_run_mode: false,
            program: None,
            freq_measurements: FreqMeasurements::new(),
        }
    }
    /// Select another part for display.
    pub fn show(&mut self, part: Part) {
        self.part = part;
    }

    pub fn toggle_auto_run_mode(&mut self) {
        self.auto_run_mode = !self.auto_run_mode
    }

    pub fn toggle_step_mode(&mut self) {
        let new_mode = match self.machine.step_mode() {
            StepMode::Real => StepMode::Assembly,
            StepMode::Assembly => StepMode::Real,
        };
        self.machine.set_step_mode(new_mode);
    }

    pub fn load_program(&mut self, path: PathBuf, bytecode: ByteCode) {
        self.machine.load(bytecode);
        self.program = Some(path);
    }

    pub fn program_path(&self) -> Option<&PathBuf> {
        self.program.as_ref()
    }

    pub fn get_frequency(&self) -> f32 {
        DEFAULT_CLK_FREQUENCY
    }

    pub fn get_measured_frequency(&self) -> f32 {
        if self.auto_run_mode {
            self.freq_measurements.get_average()
        } else {
            0.0
        }
    }

    pub fn next_cycle(&mut self) {
        self.freq_measurements.add_diff();
        self.trigger_key_clock();
    }
}

impl FreqMeasurements {
    /// Create a new empty measurement.
    pub fn new() -> Self {
        let oldest_index = 0;
        let measurements = [0.0; NUMBER_OF_MEASUREMENTS];
        let last_clk = Instant::now();
        FreqMeasurements {
            oldest_index,
            measurements,
            last_clk,
        }
    }
    /// Add a new measurement, deleting the oldest.
    /// The method returns the time since the last measurement.
    pub fn add_diff(&mut self) -> Duration {
        let clk_now = Instant::now();
        let time_since_last_measurement = clk_now - self.last_clk;
        let measurement = 1_000_000_000.0 / time_since_last_measurement.as_nanos() as f32;
        self.measurements[self.oldest_index] = measurement;
        self.oldest_index += 1;
        self.oldest_index %= NUMBER_OF_MEASUREMENTS;
        self.last_clk = clk_now;
        time_since_last_measurement
    }
    /// Return the average over the measured data.
    /// This is biased if less then NUMBER_OF_MEASUREMENTS have been pushed.
    pub fn get_average(&self) -> f32 {
        let sum: f32 = self.measurements.iter().sum();
        sum / NUMBER_OF_MEASUREMENTS as f32
    }
}

impl MachineWidget {
    /// Renders the [`OutputRegisterWidget`] correctly.
    fn render_output_registers(&self, area: Rect, buf: &mut Buffer, state: &mut MachineState) {
        // Fetch output register values
        let out_fe = state.machine.bus().output_fe();
        let out_ff = state.machine.bus().output_ff();
        // Calculate area
        let inner_area = Rect {
            width: OUTPUT_REGISTER_WIDGET_WIDTH,
            height: OUTPUT_REGISTER_WIDGET_HEIGHT,
            ..area
        };
        // Draw!
        OutputRegisterWidget.render(inner_area, buf, &mut (out_fe, out_ff));
    }
    /// Renders the [`InputRegisterWidget`] corretly.
    fn render_input_registers(&self, area: Rect, buf: &mut Buffer, state: &mut MachineState) {
        // Fetch input register values
        let in_fc = state.machine.bus().read(0xFC);
        let in_fd = state.machine.bus().read(0xFD);
        let in_fe = state.machine.bus().read(0xFE);
        let in_ff = state.machine.bus().read(0xFF);
        // Calculate area
        let inner_area = Rect {
            y: area.y + OUTPUT_REGISTER_WIDGET_HEIGHT + ONE_SPACE,
            width: INPUT_REGISTER_WIDGET_WIDTH,
            height: INPUT_REGISTER_WIDGET_HEIGHT,
            ..area
        };
        // Draw!
        InputRegisterWidget.render(inner_area, buf, &mut (in_fc, in_fd, in_fe, in_ff));
    }
    /// Renders the [`BoardInfoSidebarWidget`] correctly.
    fn render_board_info_sidebar(&self, area: Rect, buf: &mut Buffer, state: &mut MachineState) {
        if area.width > INPUT_REGISTER_WIDGET_WIDTH + BOARD_INFO_SIDEBAR_WIDGET_WIDTH {
            // Actually draw the information
            let sidebar_area = Rect {
                x: area.x + area.width - BOARD_INFO_SIDEBAR_WIDGET_WIDTH,
                width: BOARD_INFO_SIDEBAR_WIDGET_WIDTH,
                ..area
            };
            BoardInfoSidebarWidget.render(sidebar_area, buf, state)
        } else {
            // There's not enough space. Show a hint, that not everything is displayed.
            buf.set_string(area.right() - 3, area.bottom() - 1, "...", *helpers::DIMMED);
        }
    }
}

/// Draw the input register content.
///
/// # Example
/// ```
/// Inputs:
/// 00000000 00000000 00010100 00001010
///       FF       FE       FD       FC
/// ```
struct InputRegisterWidget;

impl StatefulWidget for InputRegisterWidget {
    /// Input registers FC, FD, FE, FF.
    type State = (u8, u8, u8, u8);

    fn render(self, area: Rect, buf: &mut Buffer, (fc, fd, fe, ff): &mut Self::State) {
        // Some helper constants
        const LABEL_OFFSET: u16 = 6;
        const BYTE_SPACE: u16 = BYTE_WIDTH + ONE_SPACE;
        // Make sure everything is fine. This should never fail, as
        // the interface does not draw unless a certain size is available.
        debug_assert!(area.width >= INPUT_REGISTER_WIDGET_WIDTH);
        debug_assert!(area.height >= INPUT_REGISTER_WIDGET_HEIGHT);
        // Display the "Inputs" header
        buf.set_string(area.x, area.y, "Inputs:", *helpers::DIMMED);
        // Display all the registers in binary
        render_byte(buf, area.x, area.y + 1, *ff);
        render_byte(buf, area.x + BYTE_SPACE, area.y + 1, *fe);
        render_byte(buf, area.x + 2 * BYTE_SPACE, area.y + 1, *fd);
        render_byte(buf, area.x + 3 * BYTE_SPACE, area.y + 1, *fc);
        buf.set_string(area.x + LABEL_OFFSET, area.y + 2, "FF", *helpers::DIMMED);
        buf.set_string(
            area.x + LABEL_OFFSET + BYTE_SPACE,
            area.y + 2,
            "FE",
            *helpers::DIMMED,
        );
        buf.set_string(
            area.x + LABEL_OFFSET + 2 * BYTE_SPACE,
            area.y + 2,
            "FD",
            *helpers::DIMMED,
        );
        buf.set_string(
            area.x + LABEL_OFFSET + 3 * BYTE_SPACE,
            area.y + 2,
            "FC",
            *helpers::DIMMED,
        );
    }
}

/// Draw the output register content.
///
/// # Example
/// ```
/// Outputs:
/// 00011110 00000000
///       FF       FE
/// ```
struct OutputRegisterWidget;

impl StatefulWidget for OutputRegisterWidget {
    /// Output registers FE and FF
    type State = (u8, u8);

    fn render(self, area: Rect, buf: &mut Buffer, (fe, ff): &mut Self::State) {
        // Make sure everything is fine. This should never fail, as
        // the interface does not draw unless a certain size is available.
        debug_assert!(area.width >= OUTPUT_REGISTER_WIDGET_WIDTH);
        debug_assert!(area.height >= OUTPUT_REGISTER_WIDGET_HEIGHT);
        buf.set_string(area.x, area.x, "Outputs:", *helpers::DIMMED);
        render_byte(buf, area.x, area.y + 1, *ff);
        render_byte(buf, area.x + 9, area.y + 1, *fe);
        buf.set_string(area.x + 6, area.y + 2, "FF", *helpers::DIMMED);
        buf.set_string(area.x + 15, area.y + 2, "FE", *helpers::DIMMED);
    }
}

impl StatefulWidget for MachineWidget {
    type State = MachineState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Leave some space between the border and everything else
        let area = area.inner(&Margin {
            vertical: 1,
            horizontal: 1,
        });
        // Render all the different parts interface.
        self.render_output_registers(area, buf, state);
        self.render_input_registers(area, buf, state);
        self.render_board_info_sidebar(area, buf, state);
        // Calculate remaining space
        let show_top = area.top() + SHOW_PART_START_Y_OFFSET;
        let show_area = Rect {
            y: show_top,
            height: area.bottom().saturating_sub(show_top),
            width: area.width.saturating_sub(BOARD_INFO_SIDEBAR_WIDGET_WIDTH),
            ..area
        };
        // Render the additional part

        match state.part {
            Part::Memory => {
                let memory = state.machine.bus().memory();
                MemoryWidget(memory).render(show_area, buf)
            }
            Part::RegisterBlock => {
                let registers = state.machine.registers();
                RegisterBlockWidget(registers).render(show_area, buf)
            }
        }

        // Update draw_counter
        state.draw_counter = state.draw_counter.overflowing_add(1).0;
    }
}

/// Render the given `byte` at the given `x`/`y` position.
///
/// The [`Display`] trait is used to convert the `byte` to a String.
/// If the `byte` is zero, it will be rendered with the default [`Style`].
/// If not it will be rendered in bold.
fn render_byte(buf: &mut Buffer, x: u16, y: u16, byte: u8) {
    let style = if byte == 0 {
        Style::default()
    } else {
        *helpers::BOLD
    };
    let string = byte.display();
    buf.set_string(x, y, string, style)
}

impl Deref for MachineState {
    type Target = Machine;
    fn deref(&self) -> &Self::Target {
        &self.machine
    }
}

impl DerefMut for MachineState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.machine
    }
}
