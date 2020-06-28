use colored::Colorize;
use humantime::format_duration;
use log::trace;

use std::{
    fmt,
    path::PathBuf,
    time::{Duration, Instant},
};

use crate::{
    args::RunArgs,
    error::Error,
    helpers,
    machine::{Machine, State},
};

pub struct Runner {
    /// The machine we're using.
    machine: Machine,
    /// Maximum number of cycles to emulate.
    max_cycles: usize,
    /// Number of cycles already emulated.
    emulated_cycles: usize,
    /// Program to run on the machine.
    program: PathBuf,
}

impl Runner {
    /// Create a new runner from command line arguments.
    pub fn with_args(args: &RunArgs) -> Result<Self, Error> {
        trace!("Constructing Runner..");
        let asm = helpers::read_asm_file(&args.program)?;
        Ok(Runner {
            machine: Machine::new(Some(&asm), &args.init),
            max_cycles: args.cycles,
            emulated_cycles: 0,
            program: args.program.clone(),
        })
    }
    /// Execute the runner.
    /// This results in a [`RunResults`] object that contains
    /// information about the execution.
    pub fn run(mut self) -> RunResults {
        trace!("Executing runner..");
        let before_emulation = Instant::now();
        for _ in 0..self.max_cycles {
            self.machine.clk();
            self.emulated_cycles += 1;
            if self.machine.state() != State::Running {
                trace!("Machine stopped execution");
                break;
            }
        }
        trace!("Collecting runner results..");
        RunResults::collect(self, before_emulation.elapsed())
    }
}

pub struct RunResults {
    pub program: PathBuf,
    pub out_fe: u8,
    pub out_ff: u8,
    pub state: State,
    pub emulated_cycles: usize,
    pub max_cycles: usize,
    pub time_taken: Duration,
}

impl RunResults {
    pub fn collect(runner: Runner, dur: Duration) -> Self {
        RunResults {
            out_fe: runner.machine.output_fe(),
            out_ff: runner.machine.output_ff(),
            program: runner.program,
            state: runner.machine.state(),
            emulated_cycles: runner.emulated_cycles,
            max_cycles: runner.max_cycles,
            time_taken: dur,
        }
    }
}

impl fmt::Display for RunResults {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Program: {}", self.program.to_string_lossy())?;
        writeln!(f, "Time:    {}", format_duration(self.time_taken))?;
        writeln!(
            f,
            "Cycles:  {}/{}",
            hl_if_not(self.emulated_cycles, self.max_cycles),
            self.max_cycles
        )?;
        writeln!(
            f,
            "State:   {}",
            match self.state {
                State::Running => "Running".to_owned(),
                State::Stopped => format!("{}", "Stopped".bright_yellow()),
                State::ErrorStopped => format!("{}", "Error".bright_red()),
            }
        )?;
        writeln!(f, "Output:  FE: {}", hl_if_not(self.out_fe, 0))?;
        writeln!(f, "         FF: {}", hl_if_not(self.out_ff, 0))
    }
}

fn hl_if_not<T>(val: T, cmp: T) -> String
where
    T: PartialEq + fmt::Display,
{
    if val == cmp {
        format!("{}", val)
    } else {
        format!("{}", val.to_string().bright_yellow())
    }
}
