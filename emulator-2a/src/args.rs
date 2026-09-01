use clap::{builder::TypedValueParser, Args as ClapArgs, Parser, Subcommand};
use emulator_2a_lib::{
    machine::{half_period_for_frequency, MachineConfig, State, DEFAULT_CLOCK_FREQUENCY},
    runner::{RunExpectations, RunExpectationsBuilder},
};
use log::Level;

use std::{num::ParseIntError, path::PathBuf};

/// Emulator for the Minirechner 2a microcomputer.
///
/// If run without arguments an interactive session is started.
#[derive(Debug, Parser)]
#[command(
    author = "Malte Tammena <malte.tammena@gmx.de>",
    version,
    propagate_version = true
)]
pub struct Args {
    #[command(subcommand)]
    pub subcommand: Option<SubCommand>,
    /// Increase verbosity. Can be specified multiple times to select different levels, i.e.
    /// warn, info, debug, trace. If none is specified, only errors are logged.
    #[arg(short = 'v', action = clap::ArgAction::Count)]
    pub verbosity: u8,
}

impl Args {
    /// The log [`Level`] selected by the number of `-v` occurrences.
    pub fn log_level(&self) -> Level {
        match self.verbosity {
            0 => Level::Error,
            1 => Level::Warn,
            2 => Level::Info,
            3 => Level::Debug,
            _ => Level::Trace,
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum SubCommand {
    /// Run a single emulation.
    ///
    /// The machine will be configured once according to the given flags.
    /// This happens before the emulation starts. A summary of the machine
    /// state will be printed after the emulation has run.
    ///
    /// It is sufficient to specify a program and the number of clock cycles.
    Run(RunArgs),
    /// Verify the given program's syntax.
    Verify(VerifyArgs),
    /// Run an interactive session.
    #[cfg(feature = "interactive-tui")]
    Interactive(InteractiveArgs),
}

#[derive(Debug, ClapArgs)]
pub struct RunArgs {
    #[command(flatten)]
    pub init: InitialMachineConfiguration,
    /// The path to the program to compile and run.
    ///
    /// The program will be verified before execution.
    #[arg(value_name = "PROGRAM")]
    pub program: PathBuf,
    /// The number of clock cycles to emulate.
    ///
    /// This is the _maximum_ number of clock cycles the program will
    /// run for. Emulation may be aborted before the limit is reached,
    /// i.e. if the machine halts.
    #[arg(value_name = "CYCLES")]
    pub cycles: usize,
    /// Executes a cpu reset before this cycle is executed. Can be issued multiple times.
    #[arg(long = "reset", value_name = "CYCLE")]
    pub resets: Vec<usize>,
    /// Triggers a key edge interrupt before this cycle is executed. Can be issued multiple times.
    #[arg(long = "interrupt", value_name = "CYCLE")]
    pub interrupts: Vec<usize>,
    /// Drive a square wave onto the universal I/O port UIO1.
    ///
    /// CYCLES is the half-period: the pin is held low for that many clock
    /// cycles, then high for that many, and so on, so a full period is twice
    /// CYCLES. The pin starts low, so the first edge a program sees is a
    /// rising one. Setting this does not configure the port as an input port;
    /// a program has to do that.
    #[arg(long = "uio1-square", value_name = "CYCLES")]
    pub uio1_square: Option<usize>,
    /// Drive a square wave of the given frequency onto the universal I/O port UIO1.
    ///
    /// An alternative to --uio1-square that takes a frequency instead of a
    /// cycle count. Accepts plain Hertz or a k/M prefix, i.e. `1000`, `1kHz`,
    /// `36.9kHz` or `1.5MHz`. The machine is clocked at 7.3728 MHz, so the
    /// fastest wave it can carry is 3.6864 MHz.
    #[arg(long = "uio1-freq", value_name = "FREQUENCY",
          conflicts_with = "uio1_square", value_parser = parse_frequency)]
    pub uio1_freq: Option<f64>,
    /// Drive a square wave onto the universal I/O port UIO2.
    ///
    /// See --uio1-square for more.
    #[arg(long = "uio2-square", value_name = "CYCLES")]
    pub uio2_square: Option<usize>,
    /// Drive a square wave of the given frequency onto the universal I/O port UIO2.
    ///
    /// An alternative to --uio2-square that takes a frequency instead of a
    /// cycle count. Accepts plain Hertz or a k/M prefix, i.e. `1000`, `1kHz`,
    /// `36.9kHz` or `1.5MHz`. The machine is clocked at 7.3728 MHz, so the
    /// fastest wave it can carry is 3.6864 MHz.
    ///
    /// See --uio1-freq for more.
    #[arg(long = "uio2-freq", value_name = "FREQUENCY",
          conflicts_with = "uio2_square", value_parser = parse_frequency)]
    pub uio2_freq: Option<f64>,
    /// Drive a square wave onto the universal I/O port UIO3.
    ///
    /// See --uio1-square for more.
    #[arg(long = "uio3-square", value_name = "CYCLES")]
    pub uio3_square: Option<usize>,
    /// Drive a square wave of the given frequency onto the universal I/O port UIO3.
    ///
    /// An alternative to --uio3-square that takes a frequency instead of a
    /// cycle count. Accepts plain Hertz or a k/M prefix, i.e. `1000`, `1kHz`,
    /// `36.9kHz` or `1.5MHz`. The machine is clocked at 7.3728 MHz, so the
    /// fastest wave it can carry is 3.6864 MHz.
    ///
    /// See --uio1-freq for more.
    #[arg(long = "uio3-freq", value_name = "FREQUENCY",
          conflicts_with = "uio3_square", value_parser = parse_frequency)]
    pub uio3_freq: Option<f64>,
    /// The clock frequency of the emulated machine.
    ///
    /// The emulator is driven by cycles, not by time, so this changes neither
    /// what a program does nor how many cycles it takes. It is the factor that
    /// converts cycles into time: it sets the reported emulated time, and the
    /// cycle counts that --uioN-freq works out.
    ///
    /// Accepts plain Hertz or a k/M prefix, i.e. `3686400`, `3.6864MHz`.
    #[arg(long, value_name = "FREQUENCY", value_parser = parse_frequency,
          default_value_t = DEFAULT_CLOCK_FREQUENCY)]
    pub clock: f64,
    #[command(subcommand)]
    pub verify: Option<RunVerifySubcommand>,
}

impl RunArgs {
    /// The square wave half-periods for UIO1..UIO3, in clock cycles.
    ///
    /// `--uioN-square` gives the half-period directly; `--uioN-freq` gives a
    /// frequency, which is converted here. The two conflict, so at most one of
    /// them is ever set.
    pub fn uio_squares(&self) -> [Option<usize>; 3] {
        let clock = self.clock;
        let resolve = |cycles: Option<usize>, frequency: Option<f64>| {
            cycles.or_else(|| frequency.map(|hz| half_period_for_frequency(clock, hz)))
        };
        [
            resolve(self.uio1_square, self.uio1_freq),
            resolve(self.uio2_square, self.uio2_freq),
            resolve(self.uio3_square, self.uio3_freq),
        ]
    }
}

#[derive(Debug, Clone, Subcommand)]
pub enum RunVerifySubcommand {
    /// Verify the machine state after emulation has finished.
    ///
    /// This does nothing if no expectations are given.
    /// Specify any number of expectations using the flags listed below.
    ///
    /// If any discrepance between the given expectations and the emulation
    /// results is found an error code of 1 is returned.
    Verify(RunVerifyArgs),
}

#[derive(Debug, Default, Clone, ClapArgs)]
pub struct RunVerifyArgs {
    /// The expected machine state after emulation.
    ///
    /// `stopped` expects the machine to have halted naturally because
    /// the machine executed a STOP instruction.
    ///
    /// `error` expects the machine to have halted because an error occured.
    /// This error can have different origins, i.e. a stack overflow or the
    /// execution of the 0x00 instruction. The most common causes of an error stop
    /// are a missing stackpointer initialisation or a missing program/missing jump
    /// at the end of the program.
    ///
    /// `running` expects the machine to not have halted for any reason. Of course
    /// halting and then continueing execution is valid aswell.
    #[arg(
        long,
        value_name = "STATE",
        value_parser = clap::builder::PossibleValuesParser::new(["stopped", "error", "running"])
            .map(|state| parse_state(&state)),
    )]
    pub state: Option<State>,
    /// Expected output in register FE after emulation.
    #[arg(long, value_name = "BYTE", value_parser = parse_u8_auto_radix)]
    pub fe: Option<u8>,
    /// Expected output in register FF after emulation.
    #[arg(long, value_name = "BYTE", value_parser = parse_u8_auto_radix)]
    pub ff: Option<u8>,
}

#[derive(Debug, ClapArgs)]
pub struct VerifyArgs {
    /// The path to the program to verify.
    ///
    /// The program will be verified before execution.
    #[arg(value_name = "PROGRAM")]
    pub program: PathBuf,
}

#[derive(Debug, Default, ClapArgs)]
pub struct InteractiveArgs {
    /// The path to the program to load into memory.
    ///
    /// The program will be verified before execution.
    #[arg(value_name = "PROGRAM")]
    pub program: Option<PathBuf>,
    /// The clock frequency of the emulated machine.
    ///
    /// The emulator is driven by cycles, not by time, so this changes neither
    /// what a program does nor how many cycles it takes. It is the factor that
    /// converts cycles into time: it sets the reported emulated time, and the
    /// cycle counts that --uioN-freq works out.
    ///
    /// Accepts plain Hertz or a k/M prefix, i.e. `3686400`, `3.6864MHz`.
    #[arg(long, value_name = "FREQUENCY", value_parser = parse_frequency,
          default_value_t = DEFAULT_CLOCK_FREQUENCY)]
    pub clock: f64,
    #[command(flatten)]
    pub init: InitialMachineConfiguration,
}

#[derive(Debug, Clone, Default, ClapArgs)]
pub struct InitialMachineConfiguration {
    /// Set the value of the digital input P-DI1.
    ///
    /// This input port is part of the MR2DA2 extension board.
    #[arg(long, value_name = "BYTE", default_value = "0", value_parser = parse_u8_auto_radix)]
    pub di1: u8,
    /// Set the output voltage of the temperature sensor.
    ///
    /// The temperature sensor is part of the MR2DA2 extension board.
    /// It's output voltage is fed into the comparator CP2 and powers
    /// the led D-AI2. This is equivalent to setting the analog input
    /// voltage of port P-AI2 (--ai2).
    #[arg(long, value_name = "VOLTAGE", default_value = "0")]
    pub temp: f32,
    /// Plug jumper J1 into the extension board MR2DA2.
    ///
    /// This is a universal jumper. It's current state can be read
    /// from the DA-SR status register of the MR2DA2 extension board.
    #[arg(long)]
    pub j1: bool,
    /// Plug jumper J2 into the extension board MR2DA2.
    ///
    /// This is a universal jumper. It's current state can be read
    /// from the DA-SR status register of the MR2DA2 extension board.
    #[arg(long)]
    pub j2: bool,
    /// Set the voltage at the analog input port P-AI1.
    ///
    /// The P-AI1 is part of the extension board MR2DA2. The voltage
    /// will be fed into the comparator CP1.
    #[arg(long, value_name = "VOLTAGE", default_value = "0")]
    pub ai1: f32,
    /// Set the voltage at the analog input port P-AI2.
    ///
    /// The P-AI2 is part of the extension board MR2DA2. The voltage
    /// will be fed into the comparator CP2 and power the the led D-AI2.
    /// It's effect is the same as setting the voltage of the
    /// temperature sensor (--temp)
    #[arg(long, value_name = "VOLTAGE", default_value = "0")]
    pub ai2: f32,
    /// Set the universal I/O port UIO1.
    ///
    /// The UIO1 port is located on the MR2DA2 extension board and
    /// can be used to in- or output a bit. Setting this does not
    /// configure the port as an input port. A program has to do that.
    #[arg(long)]
    pub uio1: bool,
    /// Set the universal I/O port UIO2.
    ///
    /// See UIO1 for more.
    #[arg(long)]
    pub uio2: bool,
    /// Set the universal I/O port UIO3.
    ///
    /// See UIO1 for more.
    #[arg(long)]
    pub uio3: bool,
    /// Set the content of the input register FC.
    ///
    /// This is the main way of inputing data into the program.
    #[arg(long, value_name = "BYTE", default_value = "0", value_parser = parse_u8_auto_radix)]
    pub fc: u8,
    /// Set the content of the input register FD.
    ///
    /// This is the main way of inputing data into the program.
    #[arg(long, value_name = "BYTE", default_value = "0", value_parser = parse_u8_auto_radix)]
    pub fd: u8,
    /// Set the content of the input register FE.
    ///
    /// This is the main way of inputing data into the program.
    #[arg(long, value_name = "BYTE", default_value = "0", value_parser = parse_u8_auto_radix)]
    pub fe: u8,
    /// Set the content of the input register FF.
    ///
    /// This is the main way of inputing data into the program.
    #[arg(long, value_name = "BYTE", default_value = "0", value_parser = parse_u8_auto_radix)]
    pub ff: u8,
}

impl From<InitialMachineConfiguration> for MachineConfig {
    fn from(init: InitialMachineConfiguration) -> Self {
        MachineConfig {
            analog_input1: init.ai1,
            analog_input2: init.ai2,
            digital_input1: init.di1,
            temp: init.temp,
            input_fc: init.fc,
            input_fd: init.fd,
            input_fe: init.fe,
            input_ff: init.ff,
            jumper1: init.j1,
            jumper2: init.j2,
            universal_input_output1: init.uio1,
            universal_input_output2: init.uio2,
            universal_input_output3: init.uio3,
        }
    }
}

impl From<RunVerifyArgs> for RunExpectations {
    fn from(args: RunVerifyArgs) -> Self {
        let mut expectations = RunExpectationsBuilder::default();
        if let Some(state) = args.state {
            expectations.expect_state(state);
        }
        if let Some(output_fe) = args.fe {
            expectations.expect_output_fe(output_fe);
        }
        if let Some(output_ff) = args.ff {
            expectations.expect_output_ff(output_ff);
        }
        expectations
            .build()
            .expect("BUG: Couldn't create expectations")
    }
}

/// Parse a frequency such as `1000`, `1kHz`, `36.9kHz` or `1.5MHz`.
///
/// The `Hz` is optional; a bare number is read as Hertz.
fn parse_frequency(frequency: &str) -> Result<f64, String> {
    let trimmed = frequency.trim();
    let without_unit = trimmed
        .strip_suffix("Hz")
        .or_else(|| trimmed.strip_suffix("hz"))
        .or_else(|| trimmed.strip_suffix("HZ"))
        .unwrap_or(trimmed)
        .trim_end();
    let (number, multiplier) = match without_unit.chars().last() {
        Some('k') | Some('K') => (&without_unit[..without_unit.len() - 1], 1_000.0),
        Some('M') => (&without_unit[..without_unit.len() - 1], 1_000_000.0),
        _ => (without_unit, 1.0),
    };
    let number: f64 = number
        .trim()
        .parse()
        .map_err(|_| format!("`{}` is not a frequency", frequency))?;
    let hertz = number * multiplier;
    if !(hertz > 0.0) {
        return Err(format!("`{}` is not a positive frequency", frequency));
    }
    Ok(hertz)
}

fn parse_u8_auto_radix(num: &str) -> Result<u8, ParseIntError> {
    if let Some(num) = num.strip_prefix("0b") {
        u8::from_str_radix(num, 2)
    } else if let Some(num) = num.strip_prefix("0x") {
        u8::from_str_radix(num, 16)
    } else {
        num.parse()
    }
}

/// Convert one of the values accepted by `--state` into a [`State`].
///
/// Infallible: the possible values are validated by clap before this runs.
fn parse_state(state: &str) -> State {
    match state.to_lowercase().as_str() {
        "stopped" => State::Stopped,
        "error" => State::ErrorStopped,
        "running" => State::Running,
        _ => unreachable!(),
    }
}
