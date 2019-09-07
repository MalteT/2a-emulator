//! Executor and supervisor of the emulated Machine.

use lazy_static::lazy_static;
use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::widgets::Widget;

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

pub struct Executor {
    /// The actual minirechner.
    machine: Machine,
    /// Auto run mode for emulation.
    clk_auto_run_mode: bool,
    /// Path to the currently executed program.
    program_path: Option<PathBuf>,
    /// Time since the last rising clock edge.
    last_clk_edge: Instant,
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
}

impl Executor {
    /// Initialize a new Executor.
    pub fn new() -> Self {
        let machine = Machine::new(None);
        let clk_auto_run_mode = false;
        let program_path = None;
        let last_clk_edge = Instant::now();
        let clk_period = *DEFAULT_CLK_PERIOD.deref();
        let freq_measurements = FreqMeasurements::new();
        Executor {
            machine,
            clk_auto_run_mode,
            program_path,
            last_clk_edge,
            clk_period,
            freq_measurements,
        }
    }
    /// Load a new program from the given path.
    pub fn execute<P: Into<PathBuf>>(&mut self, path: P) -> Result<(), Error> {
        self.program_path = Some(path.into());
        let program = helpers::read_asm_file(self.program_path.clone().unwrap())?;
        self.machine = Machine::new(Some(&program));
        Ok(())
    }
    /// Do necessary calculation (i.e. in auto-run-mode).
    pub fn tick(&mut self) {
        if self.clk_auto_run_mode {
            let now = Instant::now();
            let time_since_last_clk = now - self.last_clk_edge;
            self.freq_measurements
                .push(1_000_000_000.0 / time_since_last_clk.as_nanos() as f32);
            if time_since_last_clk > self.clk_period {
                self.last_clk_edge = now;
                self.machine.clk();
            }
        }
    }
    /// Emulate a rising clock edge.
    pub fn next_clk(&mut self) {
        self.machine.clk()
    }
    /// Emulate a reset.
    pub fn reset(&mut self) {
        self.machine.reset()
    }
    /// Emulate an edge interrupt.
    pub fn edge_int(&mut self) {
        self.machine.edge_int()
    }
    /// Toggle the auto-run-mode.
    pub fn toggle_auto_run_mode(&mut self) {
        self.clk_auto_run_mode = !self.clk_auto_run_mode;
    }
    /// Has the machine halted yet?
    pub fn is_halted(&self) -> bool {
        self.machine.is_halted()
    }
    /// Is auto-run-mode activated?
    pub fn is_auto_run_mode(&self) -> bool {
        self.clk_auto_run_mode
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
    /// - A tuple with a list of [`String`]s of [`Line`]s and the index of the one
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
        FreqMeasurements {
            oldest_index,
            measurements,
        }
    }
    /// Add a new measurement, deleting the oldest.
    pub fn push(&mut self, measurement: f32) {
        self.measurements[self.oldest_index] = measurement;
        self.oldest_index += 1;
        self.oldest_index %= NUMBER_OF_MEASUREMENTS;
    }
    /// Return the average over the measured data.
    /// This is biased if less then x measurements have been pushed yet.
    pub fn get_average(&self) -> f32 {
        let sum: f32 = self.measurements.iter().sum();
        sum / NUMBER_OF_MEASUREMENTS as f32
    }
}

impl Widget for Executor {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        self.machine.draw(area, buf)
    }
}
