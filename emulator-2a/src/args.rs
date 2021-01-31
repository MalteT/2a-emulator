use emulator_2a_lib::{
    machine::{MachineConfig, State},
    runner::{RunExpectations, RunExpectationsBuilder},
};
use structopt::StructOpt;

use std::{num::ParseIntError, path::PathBuf};

#[derive(Debug, StructOpt)]
#[structopt(author = "Malte Tammena <malte.tammena@gmx.de>")]
/// Emulator for the Minirechner 2a microcomputer.
///
/// If run without arguments an interactive session is started.
pub struct Args {
    #[structopt(subcommand)]
    pub subcommand: Option<SubCommand>,
}

#[derive(Debug, StructOpt)]
pub enum SubCommand {
    /// Run a single emulation.
    ///
    /// The machine will be configured once according to the given flags.
    /// This happens before the emulation starts. A summary of the machine
    /// state will be printed after the emulation has run.
    ///
    /// It is sufficient to specify a program and the number of clock cycles.
    Run(RunArgs),
    /// Run tests against a program.
    Test(TestArgs),
    /// Verify the given program's syntax.
    Verify(VerifyArgs),
    /// Run an interactive session.
    #[cfg(feature = "interactive-tui")]
    Interactive(InteractiveArgs),
}

#[derive(Debug, StructOpt)]
pub struct RunArgs {
    #[structopt(flatten)]
    pub init: InitialMachineConfiguration,
    /// The path to the program to compile and run.
    ///
    /// The program will be verified before execution.
    #[structopt(name = "PROGRAM")]
    pub program: PathBuf,
    /// The number of clock cycles to emulate.
    ///
    /// This is the _maximum_ number of clock cycles the program will
    /// run for. Emulation may be aborted before the limit is reached,
    /// i.e. if the machine halts.
    #[structopt(name = "CYCLES")]
    pub cycles: usize,
    #[structopt(subcommand)]
    pub verify: Option<RunVerifySubcommand>,
}

#[derive(Debug, Clone, StructOpt)]
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

#[derive(Debug, Clone, StructOpt)]
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
    #[structopt(long, value_name = "STATE",
                parse(from_str = parse_state),
                possible_values = &["stopped", "error", "running"])]
    pub state: Option<State>,
    /// Expected output in register FE after emulation.
    #[structopt(long, value_name = "BYTE",
                parse(try_from_str = parse_u8_auto_radix))]
    pub fe: Option<u8>,
    /// Expected output in register FF after emulation.
    #[structopt(long, value_name = "BYTE",
                parse(try_from_str = parse_u8_auto_radix))]
    pub ff: Option<u8>,
}

#[derive(Debug, StructOpt)]
pub struct TestArgs {
    /// The path to the program to test.
    ///
    /// The program will be verified before test execution.
    #[structopt(name = "PROGRAM")]
    pub program: PathBuf,
    /// The path to the test file.
    #[structopt(name = "TEST")]
    pub test: PathBuf,
}

#[derive(Debug, StructOpt)]
pub struct VerifyArgs {
    /// The path to the program to verify.
    ///
    /// The program will be verified before execution.
    #[structopt(name = "PROGRAM")]
    pub program: PathBuf,
}

#[derive(Debug, Default, StructOpt)]
pub struct InteractiveArgs {
    /// The path to the program to load into memory.
    ///
    /// The program will be verified before execution.
    #[structopt(name = "PROGRAM")]
    pub program: Option<PathBuf>,
    #[structopt(flatten)]
    pub init: InitialMachineConfiguration,
}

#[derive(Debug, Clone, Default, StructOpt)]
pub struct InitialMachineConfiguration {
    /// Set the value of the digital input P-DI1.
    ///
    /// This input port is part of the MR2DA2 extension board.
    #[structopt(long, value_name = "BYTE", default_value = "0",
                parse(try_from_str = parse_u8_auto_radix))]
    pub di1: u8,
    /// Set the output voltage of the temperature sensor.
    ///
    /// The temperature sensor is part of the MR2DA2 extension board.
    /// It's output voltage is fed into the comparator CP2 and powers
    /// the led D-AI2. This is equivalent to setting the analog input
    /// voltage of port P-AI2 (--ai2).
    #[structopt(long, value_name = "VOLTAGE", default_value = "0")]
    pub temp: f32,
    /// Plug jumper J1 into the extension board MR2DA2.
    ///
    /// This is a universal jumper. It's current state can be read
    /// from the DA-SR status register of the MR2DA2 extension board.
    #[structopt(long)]
    pub j1: bool,
    /// Plug jumper J2 into the extension board MR2DA2.
    ///
    /// This is a universal jumper. It's current state can be read
    /// from the DA-SR status register of the MR2DA2 extension board.
    #[structopt(long)]
    pub j2: bool,
    /// Set the voltage at the analog input port P-AI1.
    ///
    /// The P-AI1 is part of the extension board MR2DA2. The voltage
    /// will be fed into the comparator CP1.
    #[structopt(long, value_name = "VOLTAGE", default_value = "0")]
    pub ai1: f32,
    /// Set the voltage at the analog input port P-AI2.
    ///
    /// The P-AI2 is part of the extension board MR2DA2. The voltage
    /// will be fed into the comparator CP2 and power the the led D-AI2.
    /// It's effect is the same as setting the voltage of the
    /// temperature sensor (--temp)
    #[structopt(long, value_name = "VOLTAGE", default_value = "0")]
    pub ai2: f32,
    /// Set the universal I/O port UIO1.
    ///
    /// The UIO1 port is located on the MR2DA2 extension board and
    /// can be used to in- or output a bit. Setting this does not
    /// configure the port as an input port. A program has to do that.
    #[structopt(long)]
    pub uio1: bool,
    /// Set the universal I/O port UIO2.
    ///
    /// See UIO1 for more.
    #[structopt(long)]
    pub uio2: bool,
    /// Set the universal I/O port UIO3.
    ///
    /// See UIO1 for more.
    #[structopt(long)]
    pub uio3: bool,
    /// Set the content of the input register FC.
    ///
    /// This is the main way of inputing data into the program.
    #[structopt(long, value_name = "BYTE", default_value = "0",
                parse(try_from_str = parse_u8_auto_radix))]
    pub fc: u8,
    /// Set the content of the input register FD.
    ///
    /// This is the main way of inputing data into the program.
    #[structopt(long, value_name = "BYTE", default_value = "0",
                parse(try_from_str = parse_u8_auto_radix))]
    pub fd: u8,
    /// Set the content of the input register FE.
    ///
    /// This is the main way of inputing data into the program.
    #[structopt(long, value_name = "BYTE", default_value = "0",
                parse(try_from_str = parse_u8_auto_radix))]
    pub fe: u8,
    /// Set the content of the input register FF.
    ///
    /// This is the main way of inputing data into the program.
    #[structopt(long, value_name = "BYTE", default_value = "0",
                parse(try_from_str = parse_u8_auto_radix))]
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

fn parse_u8_auto_radix(num: &str) -> Result<u8, ParseIntError> {
    if let Some(num) = num.strip_prefix("0b") {
        u8::from_str_radix(num, 2)
    } else if let Some(num) = num.strip_prefix("0x") {
        u8::from_str_radix(num, 16)
    } else {
        u8::from_str_radix(num, 10)
    }
}

fn parse_state(state: &str) -> State {
    match state.to_lowercase().as_str() {
        "stopped" => State::Stopped,
        "error" => State::ErrorStopped,
        "running" => State::Running,
        _ => unreachable!(),
    }
}
