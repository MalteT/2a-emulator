//! Supervisor of the emulated Machine.
use emulator_2a_lib::{
    compiler::Translator,
    machine::{Machine, State},
    parser::Asm,
};
use log::trace;

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;

use crate::args::InitialMachineConfiguration;

const NUMBER_OF_MEASUREMENTS: usize = 10;
const DEFAULT_CLK_PERIOD: Duration = Duration::from_nanos((1_000.0 / 7.3728) as u64);

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
    pub final_machine_state: State,
}

impl Supervisor {
    /// Initialize a new Supervisor.
    pub fn new(conf: InitialMachineConfiguration) -> Self {
        let machine = Machine::new(conf.into());
        let clk_auto_run_mode = false;
        let clk_asm_step_mode = false;
        let clk_period = DEFAULT_CLK_PERIOD;
        let freq_measurements = FreqMeasurements::new();
        Supervisor {
            machine,
            clk_auto_run_mode,
            clk_asm_step_mode,
            clk_period,
            freq_measurements,
        }
    }
    /// Emulate the machine with the given [`EmulationParameter`].
    /// This returns a [`EmulationState`] containing all information about the emulation
    /// process.
    pub fn execute_with_parameter(
        param: EmulationParameter,
        conf: InitialMachineConfiguration,
    ) -> EmulationState {
        // Create emulation state
        let mut fs = EmulationState::new();
        // Create supervisor
        let mut sv = Supervisor::new(conf);
        sv.toggle_auto_run_mode();
        if let Some((path, asm)) = param.program {
            let bytecode = Translator::compile(&asm);
            sv.machine.load(bytecode);
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
                fs.inputs.insert(tick, *inputs);
                sv.machine.set_input_fc(inputs.0);
                sv.machine.set_input_fd(inputs.1);
                sv.machine.set_input_fe(inputs.2);
                sv.machine.set_input_ff(inputs.3);
                fs.inputs.insert(tick, *inputs);
            }
            // Check if outputs updated, and note them, if they did.
            if last_outputs != (sv.machine.bus().output_fe(), sv.machine.bus().output_ff()) {
                last_outputs = (sv.machine.bus().output_fe(), sv.machine.bus().output_ff());
                fs.outputs.insert(tick, (last_outputs.0, last_outputs.1));
            }
            // Reset the machine if needed.
            if param.resets.contains(&tick) {
                sv.machine.cpu_reset();
                fs.resets.insert(tick);
                trace!("Test: Randomly reseted machine");
            }
            // Interrupt the machine if needed.
            if param.interrupts.contains(&tick) {
                sv.machine.trigger_key_interrupt();
                fs.interrupts.insert(tick);
                trace!("Test: Set edge interrupt on machine");
            }
            // Emulate rising clk
            sv.tick();
        }
        // Add final outputs if necessary
        if last_outputs != (sv.machine.bus().output_fe(), sv.machine.bus().output_ff()) {
            last_outputs = (sv.machine.bus().output_fe(), sv.machine.bus().output_ff());
            fs.outputs
                .insert(param.ticks, (last_outputs.0, last_outputs.1));
        }
        // get the machine state at the end of execution
        fs.final_machine_state = sv.machine.state();
        fs
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
        if self.clk_asm_step_mode && self.machine().state() == State::Running {
            while self.machine.is_instruction_done() && self.machine().state() == State::Running {
                self.machine.trigger_key_clock()
            }
            while !self.machine.is_instruction_done() && self.machine().state() == State::Running {
                self.machine.trigger_key_clock()
            }
        } else {
            self.machine.trigger_key_clock()
        }
    }
    /// Toggle the auto-run-mode.
    pub fn toggle_auto_run_mode(&mut self) {
        self.clk_auto_run_mode = !self.clk_auto_run_mode;
    }
    /// Get a reference to the underlying machine.
    pub const fn machine(&self) -> &Machine {
        &self.machine
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
}

impl EmulationState {
    /// Initialize a new final state.
    fn new() -> Self {
        EmulationState {
            program: None,
            final_machine_state: State::Running,
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
        final_outputs_vec.last().map(|(_, v)| **v)
    }
}
