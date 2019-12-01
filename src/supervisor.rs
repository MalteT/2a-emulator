//! Supervisor of the emulated Machine.

use lazy_static::lazy_static;
use log::trace;
use parser2a::asm::Asm;
use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::widgets::Widget;

use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;

use crate::error::Error;
use crate::helpers;
use crate::helpers::Configuration;
use crate::machine::Machine;
use crate::machine::Part;

const NUMBER_OF_MEASUREMENTS: usize = 10;

lazy_static! {
    static ref DEFAULT_CLK_PERIOD: Duration = Duration::from_nanos((1_000.0 / 7.3728) as u64);
}

/// Supervisor of the machine.
///
/// Helps keeping emulation and displaying apart.
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
    /// Initial configuration for the machine.
    conf: Configuration,
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
#[derive(Debug, Default)]
pub struct EmulationParameter {
    /// The program path and code to execute, if any.
    pub program: Option<(PathBuf, Asm)>,
    /// The number of rising clock edges before the emulation ends.
    pub ticks: usize,
    /// The inputs that should occur at specific ticks.
    pub inputs: HashMap<usize, (u8, u8, u8, u8)>,
    /// The resets that should occur at specific ticks.
    pub resets: HashSet<usize>,
    /// The interrupts that should occur at specific ticks.
    pub interrupts: HashSet<usize>,
}

/// State of an emulation, containing all details about the process.
#[derive(Debug)]
pub struct EmulationState {
    /// The program, which was emulated, if any.
    pub program: Option<(PathBuf, Asm)>,
    /// The inputs with the given tick at which they occured.
    pub inputs: HashMap<usize, (u8, u8, u8, u8)>,
    /// The outputs with the given tick at which they occured.
    pub outputs: HashMap<usize, (u8, u8)>,
    /// The resets with the given tick at which they occured.
    pub resets: HashSet<usize>,
    /// The interrupts with the given tick at which they occured.
    pub interrupts: HashSet<usize>,
    /// The final state of the machine.
    pub final_machine_state: MachineState,
}

/// Machine state after execution.
#[derive(Debug)]
pub enum MachineState {
    Stopped,
    ErrorStopped,
    Running,
}

impl Supervisor {
    /// Initialize a new Supervisor.
    pub fn new(conf: &Configuration) -> Self {
        let machine = Machine::new(None, conf);
        let clk_auto_run_mode = false;
        let clk_asm_step_mode = false;
        let program_path = None;
        let clk_period = *DEFAULT_CLK_PERIOD.deref();
        let freq_measurements = FreqMeasurements::new();
        let conf = conf.clone();
        Supervisor {
            machine,
            clk_auto_run_mode,
            clk_asm_step_mode,
            program_path,
            clk_period,
            freq_measurements,
            conf,
        }
    }
    /// Emulate the machine with the given [`EmulationParameter`].
    /// This returns a [`EmulationState`] containing all information about the emulation
    /// process.
    pub fn execute_with_parameter(
        param: EmulationParameter,
        conf: &Configuration,
    ) -> EmulationState {
        // Create emulation state
        let mut fs = EmulationState::new();
        // Create supervisor
        let mut sv = Supervisor::new(conf);
        sv.toggle_auto_run_mode();
        if let Some((path, asm)) = param.program {
            sv.program_path = Some(path.clone());
            sv.machine = Machine::new(Some(&asm), conf);
            fs.program = Some((path, asm));
        }
        // Remember initial outputs
        fs.outputs.insert(0, (0, 0));
        // Remember input/outputs
        let mut last_outputs = (0, 0);
        // MAIN LOOP
        for tick in 0..=param.ticks {
            // Input the inputs accordingly.
            if param.inputs.contains_key(&tick) {
                let inputs = param.inputs.get(&tick).unwrap();
                fs.inputs.insert(tick, inputs.clone());
                sv.machine.input_fc(inputs.0);
                sv.machine.input_fd(inputs.1);
                sv.machine.input_fe(inputs.2);
                sv.machine.input_ff(inputs.3);
                fs.inputs.insert(tick, *inputs);
            }
            // Check if outputs updated, and note them, if they did.
            if last_outputs != (sv.machine.output_fe(), sv.machine.output_ff()) {
                last_outputs = (sv.machine.output_fe(), sv.machine.output_ff());
                fs.outputs.insert(tick, (last_outputs.0, last_outputs.1));
            }
            // Reset the machine if needed.
            if param.resets.contains(&tick) {
                sv.machine.reset();
                fs.resets.insert(tick);
                trace!("Test: Randomly reseted machine");
            }
            // Interrupt the machine if needed.
            if param.interrupts.contains(&tick) {
                sv.machine.key_edge_int();
                fs.interrupts.insert(tick);
                trace!("Test: Set edge interrupt on machine");
            }
            // Emulate rising clk
            sv.tick();
        }
        // Add final outputs if necessary
        if last_outputs != (sv.machine.output_fe(), sv.machine.output_ff()) {
            last_outputs = (sv.machine.output_fe(), sv.machine.output_ff());
            fs.outputs
                .insert(param.ticks, (last_outputs.0, last_outputs.1));
        }
        // get the machine state at the end of execution
        let final_machine_state = if sv.machine.is_error_stopped() {
            MachineState::ErrorStopped
        } else if sv.machine.is_stopped() {
            MachineState::Stopped
        } else {
            MachineState::Running
        };
        fs.final_machine_state = final_machine_state;
        fs
    }
    /// Load a new program from the given path.
    /// Resets the machine.
    pub fn load_program<P: Into<PathBuf>>(&mut self, path: P) -> Result<(), Error> {
        self.program_path = Some(path.into());
        let program = helpers::read_asm_file(self.program_path.clone().unwrap())?;
        self.machine = Machine::new(Some(&program), &self.conf);
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
    /// Set input register FC.
    pub fn input_fc(&mut self, byte: u8) {
        self.machine.input_fc(byte)
    }
    /// Set input register FD.
    pub fn input_fd(&mut self, byte: u8) {
        self.machine.input_fd(byte)
    }
    /// Set input register FE.
    pub fn input_fe(&mut self, byte: u8) {
        self.machine.input_fe(byte)
    }
    /// Set input register FF.
    pub fn input_ff(&mut self, byte: u8) {
        self.machine.input_ff(byte)
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
    /// Set the 8-bit input port.
    pub fn set_irg(&mut self, value: u8) {
        self.machine.bus.board.set_irg(value);
    }
    /// Set the temperature value.
    pub fn set_temp(&mut self, value: f32) {
        self.machine.bus.board.set_temp(value);
    }
    /// Set the jumper J1.
    ///
    /// - `true` => Plugged in.
    /// - `false` => Unplugged.
    pub fn set_j1(&mut self, plugged: bool) {
        self.machine.bus.board.set_j1(plugged);
    }
    /// Set the jumper J2.
    ///
    /// - `true` => Plugged in.
    /// - `false` => Unplugged.
    pub fn set_j2(&mut self, plugged: bool) {
        self.machine.bus.board.set_j2(plugged);
    }
    /// Set the analog input I1.
    pub fn set_i1(&mut self, value: f32) {
        self.machine.bus.board.set_i1(value);
    }
    /// Set the analog input I2.
    pub fn set_i2(&mut self, value: f32) {
        self.machine.bus.board.set_i2(value);
    }
    /// Set the Universal Input/Output UIO1.
    pub fn set_uio1(&mut self, value: bool) {
        self.machine.bus.board.set_uio1(value);
    }
    /// Set the Universal Input/Output UIO2.
    pub fn set_uio2(&mut self, value: bool) {
        self.machine.bus.board.set_uio2(value);
    }
    /// Set the Universal Input/Output UIO3.
    pub fn set_uio3(&mut self, value: bool) {
        self.machine.bus.board.set_uio3(value);
    }
    /// Is an asm program loaded?
    pub fn is_program_loaded(&self) -> bool {
        self.program_path.is_some()
    }
    /// Select the part to show in the TUI.
    pub fn show(&mut self, part: Part) {
        self.machine.show(part);
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

impl EmulationState {
    /// Initialize a new final state.
    fn new() -> Self {
        EmulationState {
            program: None,
            final_machine_state: MachineState::Running,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            interrupts: HashSet::new(),
            resets: HashSet::new(),
        }
    }
    /// Get the last outputs noted in the state.
    pub fn final_outputs(&self) -> Option<(u8, u8)> {
        let mut final_outputs_vec: Vec<(_, _)> = self.outputs.iter().collect();
        final_outputs_vec.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));
        final_outputs_vec.last().map(|(_, v)| *v.clone())
    }
}

impl Widget for Supervisor {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        self.machine.draw(area, buf)
    }
}
