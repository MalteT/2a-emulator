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

fn parse_u8_auto_radix(num: &str) -> Result<u8, ParseIntError> {
    if num.starts_with("0b") {
        u8::from_str_radix(&num[2..], 2)
    } else if num.starts_with("0x") {
        u8::from_str_radix(&num[2..], 16)
    } else {
        u8::from_str_radix(num, 10)
    }
}
