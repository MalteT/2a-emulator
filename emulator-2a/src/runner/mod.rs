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
        .with_resets(args.resets.clone())
        .with_interrupts(args.interrupts.clone())
        .with_uio_squares(args.uio_squares())
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

#[cfg(test)]
mod tests {
    use crate::args::{InitialMachineConfiguration, RunVerifyArgs};

    use super::*;

    #[test]
    fn flags_are_not_ignored_if_program_is_given() {
        let run_args = RunArgs {
            init: InitialMachineConfiguration {
                fc: 1,
                fd: 2,
                fe: 3,
                ff: 4,
                ..Default::default()
            },
            program: "../testing/programs/26-specific-input.asm".into(),
            cycles: 1000,
            resets: vec![],
            interrupts: vec![],
            uio1_square: None,
            uio2_square: None,
            uio3_square: None,
            uio1_freq: None,
            uio2_freq: None,
            uio3_freq: None,
            clock: emulator_2a_lib::machine::DEFAULT_CLOCK_FREQUENCY,
            verify: Some(RunVerifySubcommand::Verify(RunVerifyArgs {
                state: Some(State::Running),
                ..Default::default()
            })),
        };
        execute_runner_with_args_and_print_results(&run_args).unwrap();
    }

    #[test]
    fn uio_square_wave_drives_the_pin() {
        // The program counts rising edges on UIO1 into the output register FF
        // by polling.
        let run_args = RunArgs {
            init: InitialMachineConfiguration::default(),
            program: "../testing/programs/27-uio1-polling-counter.asm".into(),
            cycles: 20_000,
            resets: vec![],
            interrupts: vec![],
            uio1_square: Some(100),
            uio2_square: None,
            uio3_square: None,
            uio1_freq: None,
            uio2_freq: None,
            uio3_freq: None,
            clock: emulator_2a_lib::machine::DEFAULT_CLOCK_FREQUENCY,
            verify: Some(RunVerifySubcommand::Verify(RunVerifyArgs {
                ff: Some(100),
                ..Default::default()
            })),
        };
        execute_runner_with_args_and_print_results(&run_args).unwrap();
    }

    #[test]
    fn the_clock_frequency_scales_a_requested_wave_frequency() {
        let mut args = RunArgs {
            init: InitialMachineConfiguration::default(),
            program: "../testing/programs/07-minimal.asm".into(),
            cycles: 1,
            resets: vec![],
            interrupts: vec![],
            uio1_square: None,
            uio2_square: None,
            uio3_square: None,
            uio1_freq: Some(1_000.0),
            uio2_freq: None,
            uio3_freq: None,
            clock: 7_372_800.0,
            verify: None,
        };
        // 7372800 / (2 * 1000)
        assert_eq!(args.uio_squares()[0], Some(3686));
        // Halving the clock halves the cycle count for the same wave.
        args.clock = 3_686_400.0;
        assert_eq!(args.uio_squares()[0], Some(1843));
        // An explicit cycle count is not affected by the clock at all.
        args.uio1_square = Some(500);
        assert_eq!(args.uio_squares()[0], Some(500));
    }
}
