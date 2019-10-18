//! Supervisor of the emulated Machine.

use lazy_static::lazy_static;
use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::widgets::Widget;
use parser2a::asm::Asm;

use std::ops::Deref;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;

use crate::error::Error;
use crate::helpers;
use crate::machine::Machine;

const NUMBER_OF_MEASUREMENTS: usize = 10;

lazy_static! {
    static ref DEFAULT_CLK_PERIOD: Duration = Duration::from_nanos((1_000.0 / 7.3728) as u64);
}

pub struct Supervisor {
    /// The actual minirechner.
    machine: Machine,
    /// Auto run mode for emulation.
    clk_auto_run_mode: bool,
    /// Asm step mode for emulation.
    clk_asm_step_mode: bool,
    /// Path to the currently executed program.
    program_path: Option<PathBuf>,
    /// Time between two rising clock edges.
    clk_period: Duration,
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

/// Assortment of parameters used for emulation.
pub struct EmulationParameter {
    /// The program path and code to execute, if any.
    program: Option<(PathBuf, Asm)>,
    /// The number of rising clock edges before the emulation ends.
    ticks: usize,
    /// The inputs that should occur at specific ticks.
    inputs: Vec<(usize, u8, u8, u8, u8)>,
    // TODO: Interrupts
}

/// Final state of an emulation, containing all details about the process.
pub struct FinalState {
    /// The program, which was emulated, if any.
    program: Option<(PathBuf, Asm)>,
    /// The inputs with the given tick at which they occured.
    inputs: Vec<(usize, u8, u8, u8, u8)>,
    /// The outputs with the given tick at which they occured.
    outputs: Vec<(usize, u8, u8)>,
    // TODO: Interrupts
}

impl Supervisor {
    /// Initialize a new Supervisor.
    pub fn new() -> Self {
        let machine = Machine::new(None);
        let clk_auto_run_mode = false;
        let clk_asm_step_mode = false;
        let program_path = None;
        let clk_period = *DEFAULT_CLK_PERIOD.deref();
        let freq_measurements = FreqMeasurements::new();
        Supervisor {
            machine,
            clk_auto_run_mode,
            clk_asm_step_mode,
            program_path,
            clk_period,
            freq_measurements,
        }
    }
    /// Emulate the machine with the given [`Parameter`].
    /// This returns a [`FinalState`] containing all information about the emulation
    /// process.
    pub fn execute_with_parameter(param: EmulationParameter) -> FinalState {
        // Create final state
        let mut fs = FinalState::new();
        // Create supervisor
        let mut sv = Supervisor::new();
        sv.toggle_auto_run_mode();
        if let Some((path, asm)) = param.program {
            sv.program_path = Some(path.clone());
            sv.machine = Machine::new(Some(&asm));
            fs.program = Some((path, asm));
        }
        // Remember input/outputs
        let mut input_index = 0;
        let mut last_outputs = (0, 0);
        // MAIN LOOP
        for tick in 0..param.ticks + 1 {
            // Input the inputs accordingly.
            if param.inputs[input_index].0 == tick {
                let inputs = param.inputs[input_index];
                fs.inputs.push(inputs);
                sv.machine.input_fc(inputs.1);
                sv.machine.input_fd(inputs.2);
                sv.machine.input_fe(inputs.3);
                sv.machine.input_ff(inputs.4);
                if param.inputs.len() > input_index + 1 {
                    input_index += 1;
                }
            }
            // Check if outputs updated, and note them, if they did.
            if last_outputs != (sv.machine.output_fe(), sv.machine.output_ff()) {
                last_outputs = (sv.machine.output_fe(), sv.machine.output_ff());
                fs.outputs.push((tick, last_outputs.0, last_outputs.1));
            }
            // Emulate rising clk
            sv.tick();
        }
        fs
    }
    /// Load a new program from the given path.
    /// Resets the machine.
    pub fn load_program<P: Into<PathBuf>>(&mut self, path: P) -> Result<(), Error> {
        self.program_path = Some(path.into());
        let program = helpers::read_asm_file(self.program_path.clone().unwrap())?;
        self.machine = Machine::new(Some(&program));
        Ok(())
    }
    /// Do necessary calculation (i.e. in auto-run-mode).
    pub fn tick(&mut self) {
        if self.clk_auto_run_mode {
            let time_since_last_clk = self.freq_measurements.add_diff();
            if time_since_last_clk > self.clk_period {
                self.next_clk();
            }
        }
    }
    /// Emulate a rising clock edge.
    pub fn next_clk(&mut self) {
        if self.clk_asm_step_mode && !(self.is_stopped() || self.is_error_stopped()) {
            while self.machine.is_instruction_done() {
                self.machine.clk()
            }
            while !self.machine.is_instruction_done() {
                self.machine.clk()
            }
        } else {
            self.machine.clk()
        }
    }
    /// Toggle between single step and asm step modes.
    pub fn toggle_asm_step_mode(&mut self) {
        self.clk_asm_step_mode = !self.clk_asm_step_mode;
    }
    /// Emulate a reset.
    pub fn reset(&mut self) {
        self.machine.reset()
    }
    /// Emulate an edge interrupt.
    pub fn key_edge_int(&mut self) {
        self.machine.key_edge_int()
    }
    /// Toggle the auto-run-mode.
    pub fn toggle_auto_run_mode(&mut self) {
        self.clk_auto_run_mode = !self.clk_auto_run_mode;
    }
    /// Is key edge interrupt enabled?
    pub fn is_key_edge_int_enabled(&self) -> bool {
        self.machine.is_key_edge_int_enabled()
    }
    /// Was the machine stopped yet?
    pub fn is_stopped(&self) -> bool {
        self.machine.is_stopped()
    }
    /// Was the machine error stopped yet?
    pub fn is_error_stopped(&self) -> bool {
        self.machine.is_error_stopped()
    }
    /// Continue the machine after a stop.
    pub fn continue_from_stop(&mut self) {
        self.machine.continue_from_stop()
    }
    /// Is auto-run-mode activated?
    pub fn is_auto_run_mode(&self) -> bool {
        self.clk_auto_run_mode
    }
    /// Is asm-step-mode activated?
    pub fn is_asm_step_mode(&self) -> bool {
        self.clk_asm_step_mode
    }
    /// Get the frequency settings.
    pub fn get_frequency(&self) -> f32 {
        1_000_000_000.0 / self.clk_period.as_nanos() as f32
    }
    /// Returns whether the measured frequency is drastically below the frequency setting.
    pub fn is_at_full_capacity(&self) -> bool {
        self.get_measured_frequency() < self.get_frequency() * 0.9
    }
    /// Get the currently executed lines of the program.
    ///
    /// # Arguments
    /// - `context` The amount of lines before and after the currently executed line.
    ///
    /// # Returns
    /// - A tuple with a list of [`String`]s of asm lines and the index of the one
    /// currently executed by the machine.
    pub fn get_current_lines(&self, context: isize) -> (usize, Vec<&String>) {
        self.machine.get_current_lines(context)
    }
    /// Get the currently running programs path.
    pub fn get_program_path(&self) -> &Option<PathBuf> {
        &self.program_path
    }
    /// Get the average measured frequency of the machine during the last x clock edges.
    /// This returns `0.0` if the machine is *not* in auto-run-mode.
    pub fn get_measured_frequency(&self) -> f32 {
        if self.clk_auto_run_mode {
            self.freq_measurements.get_average()
        } else {
            0.0
        }
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
    /// This is biased if less then x measurements have been pushed yet.
    pub fn get_average(&self) -> f32 {
        let sum: f32 = self.measurements.iter().sum();
        sum / NUMBER_OF_MEASUREMENTS as f32
    }
}

impl FinalState {
    /// Initialize a new final state.
    fn new() -> Self {
        FinalState {
            program: None,
            inputs: vec![],
            outputs: vec![(0, 0, 0)],
        }
    }
}

impl Widget for Supervisor {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        self.machine.draw(area, buf)
    }
}
