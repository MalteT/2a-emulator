use colored::Colorize;
use emulator_2a_lib::{
    compiler::Translator,
    machine::{Machine, State},
};
use humantime::format_duration;
use log::trace;

use std::{
    fmt,
    path::PathBuf,
    time::{Duration, Instant},
};

use crate::{
    args::{RunArgs, RunVerifyArgs, RunVerifySubcommand},
    error::Error,
    helpers,
};

pub struct Runner<'a> {
    /// The machine we're using.
    machine: Machine,
    /// Maximum number of cycles to emulate.
    max_cycles: usize,
    /// Number of cycles already emulated.
    emulated_cycles: usize,
    /// Program to run on the machine.
    program: PathBuf,
    /// Command line arguments.
    args: &'a RunArgs,
}

impl<'a> Runner<'a> {
    /// Create a new runner from command line arguments.
    pub fn with_args(args: &'a RunArgs) -> Result<Self, Error> {
        trace!("Constructing Runner..");
        let asm = helpers::read_asm_file(&args.program)?;
        let bytecode = Translator::compile(&asm);
        let mut machine = Machine::new(args.init.clone().into());
        machine.load(bytecode);
        Ok(Runner {
            machine,
            max_cycles: args.cycles,
            emulated_cycles: 0,
            program: args.program.clone(),
            args,
        })
    }
    /// Execute the runner.
    ///
    /// This executes the runner and checks all verifications.
    pub fn run(mut self) -> Result<(), Error> {
        trace!("Executing runner..");
        let before_emulation = Instant::now();
        for _ in 0..self.max_cycles {
            self.machine.trigger_key_clock();
            self.emulated_cycles += 1;
            if self.machine.state() != State::Running {
                trace!("Machine stopped execution");
                break;
            }
        }
        trace!("Collecting runner results..");
        RunResults::collect_and_verify(&self, before_emulation.elapsed())
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
    /// Collects the results of the emulation.
    pub fn collect(runner: &Runner, dur: Duration) -> Self {
        let bus = runner.machine.bus();
        RunResults {
            out_fe: bus.output_fe(),
            out_ff: bus.output_ff(),
            program: runner.program.clone(),
            state: runner.machine.state(),
            emulated_cycles: runner.emulated_cycles,
            max_cycles: runner.max_cycles,
            time_taken: dur,
        }
    }
    /// Verify that all expectations match this result.
    ///
    /// All expectations given by [`RunVerifyArgs`] will be checked
    /// against this [`RunResults`] object.
    ///
    /// This function will return an [`Error`] if any discrepance
    /// is found between the objects.
    pub fn verify(&self, args: &RunVerifyArgs) -> Result<(), Error> {
        if !satisfies(&args.state, &self.state) {
            Err(Error::RunVerificationFailed("State".into()))
        } else if !satisfies(&args.fe, &self.out_fe) {
            Err(Error::RunVerificationFailed("Output register FE".into()))
        } else if !satisfies(&args.ff, &self.out_ff) {
            Err(Error::RunVerificationFailed("Output register FF".into()))
        } else {
            Ok(())
        }
    }
    /// Collect the results and verify them.
    pub fn collect_and_verify(runner: &Runner, dur: Duration) -> Result<(), Error> {
        let results = RunResults::collect(&runner, dur);
        if let Some(RunVerifySubcommand::Verify(args)) = &runner.args.verify {
            println!("{}", results);
            results.verify(args)
        } else {
            // Well, everything is already correct then
            println!("{}", results);
            Ok(())
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
            hl_if_not(&self.emulated_cycles, &self.max_cycles),
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
        writeln!(f, "Output:  FE: {}", hl_if_not(&self.out_fe, &0))?;
        writeln!(f, "         FF: {}", hl_if_not(&self.out_ff, &0))
    }
}

fn hl_if_not<T>(val: &T, cmp: &T) -> String
where
    T: PartialEq + fmt::Display,
{
    if *val == *cmp {
        format!("{}", val)
    } else {
        format!("{}", val.to_string().bright_yellow())
    }
}

/// Returns if `opt` is [`Some`] and equals `cmp` or if `opt` is [`None`].
fn satisfies<T: PartialEq>(opt: &Option<T>, cmp: &T) -> bool {
    match opt {
        None => true,
        Some(val) if val == cmp => true,
        _ => false,
    }
}
