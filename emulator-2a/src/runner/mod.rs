use colored::Colorize;
use emulator_2a_lib::{
    machine::State,
    runner::{RunExpectations, RunResults, RunnerConfigBuilder, VerificationError},
};
use humantime::format_duration;
use log::trace;

use std::{fmt, fs::read_to_string};

use crate::{
    args::{RunArgs, RunVerifySubcommand},
    error::Error,
};

pub fn execute_runner_with_args_and_print_results(args: &RunArgs) -> Result<(), Error> {
    trace!("Constructing Runner..");
    let program = read_to_string(&args.program)?;
    let config = RunnerConfigBuilder::default()
        .with_machine_config(args.init.clone().into())
        .with_max_cycles(args.cycles)
        .with_program(&program)
        .build()
        .expect("Failed to create RunnerConfig");
    trace!("Running Runner..");
    let results = config.run()?;
    let status: Result<(), VerificationError> =
        if let Some(RunVerifySubcommand::Verify(verify_args)) = args.verify.clone() {
            trace!("Constructing expectations..");
            let expectations: RunExpectations = verify_args.into();
            expectations.verify(&results)
        } else {
            Ok(())
        };
    print_run_results(args, &results);
    Ok(status?)
}

fn print_run_results(args: &RunArgs, res: &RunResults) {
    trace!("Printing Runner results..");
    println!("Program: {}", args.program.to_string_lossy());
    println!("Time:    {}", format_duration(res.time_taken));
    println!(
        "Cycles:  {}/{}",
        hl_if_not(&res.emulated_cycles, &res.config.max_cycles),
        res.config.max_cycles
    );
    println!(
        "State:   {}",
        match res.machine.state() {
            State::Running => "Running".to_owned(),
            State::Stopped => format!("{}", "Stopped".bright_yellow()),
            State::ErrorStopped => format!("{}", "Error".bright_red()),
        }
    );
    println!(
        "Output:  FE: {}",
        hl_if_not(&res.machine.bus().output_fe(), &0)
    );
    println!(
        "         FF: {}",
        hl_if_not(&res.machine.bus().output_ff(), &0)
    );
    println!()
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
