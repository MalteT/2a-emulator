use colored::*;
use log::{info, trace};
use parser2a::asm::Asm;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use rand::{random, thread_rng, Rng};

use std::fs::read_to_string;
use std::path::PathBuf;

use crate::error::Error;
use crate::helpers;
use crate::helpers::Configuration;
use crate::supervisor::{EmulationParameter, MachineState, Supervisor};

#[derive(Debug, Parser)]
#[grammar = "../static/tests.pest"]
struct TestParser;

#[derive(Debug, Clone, Copy)]
pub enum Input {
    Random,
    Number(u8),
}

#[derive(Debug)]
pub enum Setting {
    RandomInput,
    RandomReset,
    RandomInterrupt,
    Interrupt,
    InputFc(Input),
    InputFd(Input),
    InputFe(Input),
    InputFf(Input),
}

#[derive(Debug)]
pub enum Expectation {
    Stop,
    NoStop,
    ErrorStop,
    NoErrorStop,
    OutputFe(u8),
    OutputFf(u8),
}

#[derive(Debug)]
pub struct Test {
    name: String,
    ticks: usize,
    settings: Vec<Setting>,
    expectations: Vec<Expectation>,
}

#[derive(Debug)]
pub struct TestFile {
    tests: Vec<Test>,
}

impl TestFile {
    pub fn parse<P: Into<PathBuf>>(path: P) -> Result<Self, Error> {
        let path: PathBuf = path.into();
        let testfile = read_to_string(&path)?;
        // Parse file
        let parsed = TestParser::parse(Rule::file, &testfile)?;

        // Translate the parsed tree
        let mut tests = vec![];
        for test in parsed {
            match test.as_rule() {
                Rule::test => tests.push(Test::parse(test)),
                Rule::EOI => {}
                _ => unreachable!(),
            }
        }
        trace!("Parsed test file from path {:?}", path);
        Ok(TestFile { tests })
    }
    pub fn execute_against<P: Into<PathBuf>>(
        &self,
        path: P,
        conf: &Configuration,
    ) -> Result<(), Error> {
        let path: PathBuf = path.into();
        let asm = helpers::read_asm_file(&path)?;
        let mut res = Ok(());
        for test in &self.tests {
            trace!("Executing test {:?} for {:?}", test.name, path);
            res = match test.execute_against(&path, asm.clone(), conf) {
                Err(e) => {
                    eprintln!(" {} {}", "=>".bright_red(), e);
                    Err(e)
                }
                Ok(()) => res,
            };
        }
        res
    }
}

impl Test {
    /// Execute the test.
    pub fn execute_against<P: Into<PathBuf>>(
        &self,
        path: P,
        asm: Asm,
        conf: &Configuration,
    ) -> Result<(), Error> {
        let path: PathBuf = path.into();
        let mut rng = thread_rng();

        // Initialize stuff
        let mut ep = EmulationParameter::default();
        ep.program = Some((path.clone(), asm));

        ep.ticks = self.ticks;
        let mut initial_input = (0, 0, 0, 0);
        for setting in &self.settings {
            match setting {
                Setting::InputFc(nr) => initial_input.0 = nr.to_num(),
                Setting::InputFd(nr) => initial_input.1 = nr.to_num(),
                Setting::InputFe(nr) => initial_input.2 = nr.to_num(),
                Setting::InputFf(nr) => initial_input.3 = nr.to_num(),
                Setting::RandomInput => {
                    initial_input.0 = random();
                    initial_input.1 = random();
                    initial_input.2 = random();
                    initial_input.3 = random();
                }
                Setting::RandomReset => {
                    for _ in 1..rng.gen_range(1, self.ticks / 200) {
                        ep.resets.insert(rng.gen_range(1, self.ticks));
                    }
                }
                Setting::RandomInterrupt => {
                    for _ in 1..rng.gen_range(1, self.ticks / 200) {
                        ep.interrupts.insert(rng.gen_range(1, self.ticks));
                    }
                }
                Setting::Interrupt => {
                    ep.interrupts.insert(self.ticks / 2);
                }
            }
        }

        // Set initial inputs
        ep.inputs.insert(0, initial_input);

        // Run the emulation
        let final_state = Supervisor::execute_with_parameter(ep, conf);
        let final_outputs = final_state.final_outputs();

        // Verify
        trace!("Verifying test {:?} for {:?}", self.name, path);
        for expectation in &self.expectations {
            match expectation {
                Expectation::Stop => match final_state.final_machine_state {
                    MachineState::ErrorStopped => {}
                    MachineState::Stopped => {}
                    MachineState::Running => return self.create_error("Machine did not stop!"),
                },
                Expectation::NoStop => match final_state.final_machine_state {
                    MachineState::ErrorStopped | MachineState::Stopped => {
                        return self.create_error("Machine stopped!")
                    }
                    MachineState::Running => {}
                },
                Expectation::ErrorStop => match final_state.final_machine_state {
                    MachineState::ErrorStopped => {}
                    MachineState::Stopped | MachineState::Running => {
                        return self.create_error("Machine did not error stop!")
                    }
                },
                Expectation::NoErrorStop => match final_state.final_machine_state {
                    MachineState::ErrorStopped => {
                        return self.create_error("Machine error stopped!")
                    }
                    MachineState::Stopped | MachineState::Running => {}
                },
                Expectation::OutputFe(nr) => {
                    if final_outputs.is_some() && final_outputs.unwrap().0 != *nr {
                        return self.create_error(&format!(
                            "Different output on FE: {} != {}",
                            nr,
                            final_outputs.unwrap().0
                        ));
                    }
                }
                Expectation::OutputFf(nr) => {
                    if final_outputs.is_some() && final_outputs.unwrap().1 != *nr {
                        return self.create_error(&format!(
                            "Different output on FF: {} != {}",
                            nr,
                            final_outputs.unwrap().1
                        ));
                    }
                }
            };
        }
        info!("Test {:?} for {:?} was successful", self.name, path);
        Ok(())
    }
    /// Wraps the given &str into an `Err(Error)` for easy error creation.
    fn create_error(&self, s: &str) -> Result<(), Error> {
        Err(Error::TestFailed(self.name.clone(), s.into()))
    }
    /// Parse a test from the given Pest Pair.
    fn parse(pair: Pair<Rule>) -> Self {
        let mut name = "".into();
        let mut ticks = 10_000;
        let mut settings = vec![];
        let mut expectations = vec![];

        for part in pair.into_inner() {
            match part.as_rule() {
                Rule::test_name => {
                    let s = part.as_str();
                    name = s[1..s.len() - 1].into();
                }
                Rule::with_block => settings = Test::parse_settings(part),
                Rule::for_block => ticks = Test::parse_ticks(part),
                Rule::expect_block => expectations = Test::parse_expectations(part),
                _ => unreachable!(),
            }
        }

        Test {
            name,
            ticks,
            settings,
            expectations,
        }
    }
    /// Parse with_block from given Pest Pair.
    fn parse_settings(pair: Pair<Rule>) -> Vec<Setting> {
        let mut ret = vec![];
        for expectation in pair.into_inner() {
            let expectation = match expectation.as_rule() {
                Rule::random_input => Setting::RandomInput,
                Rule::random_reset => Setting::RandomReset,
                Rule::random_interrupt => Setting::RandomInterrupt,
                Rule::interrupt => Setting::Interrupt,
                Rule::fc => Test::parse_input_setting(expectation),
                Rule::fd => Test::parse_input_setting(expectation),
                Rule::fe => Test::parse_input_setting(expectation),
                Rule::ff => Test::parse_input_setting(expectation),
                _ => unreachable!(),
            };
            ret.push(expectation);
        }
        ret
    }
    /// Parse fc/fd/fe/ff from given Pest Pair into [`Setting`].
    fn parse_input_setting(pair: Pair<Rule>) -> Setting {
        let inp = pair.clone().into_inner().next().expect("Infallible");
        let inp = match inp.as_rule() {
            Rule::random => Input::Random,
            Rule::hex_digit => {
                let raw = &inp.as_str()[2..];
                let number = u8::from_str_radix(raw, 16).expect("Infallible");
                Input::Number(number)
            }
            _ => unreachable!(),
        };
        match pair.as_rule() {
            Rule::fc => Setting::InputFc(inp),
            Rule::fd => Setting::InputFd(inp),
            Rule::fe => Setting::InputFe(inp),
            Rule::ff => Setting::InputFf(inp),
            _ => unreachable!(),
        }
    }
    /// Parse ticks from Pest Pair into [`Setting`].
    fn parse_ticks(pair: Pair<Rule>) -> usize {
        let raw = pair.into_inner().next().expect("Infallible").as_str();
        usize::from_str_radix(raw, 10).expect("Infallible")
    }
    /// Parse expectation from Pest Pair into [`Setting`].
    fn parse_expectations(pair: Pair<Rule>) -> Vec<Expectation> {
        let mut ret = vec![];
        for pair in pair.into_inner() {
            let expectation = match pair.as_rule() {
                Rule::stop => Expectation::Stop,
                Rule::no_stop => Expectation::NoStop,
                Rule::error_stop => Expectation::ErrorStop,
                Rule::no_error_stop => Expectation::NoErrorStop,
                Rule::out_fe => {
                    let raw = &pair.into_inner().as_str()[2..];
                    let number = u8::from_str_radix(raw, 16).expect("Infallible");
                    Expectation::OutputFe(number)
                }
                Rule::out_ff => {
                    let raw = &pair.into_inner().as_str()[2..];
                    let number = u8::from_str_radix(raw, 16).expect("Infallible");
                    Expectation::OutputFf(number)
                }
                _ => panic!("OH NO! {:#?}", pair),
            };
            ret.push(expectation);
        }
        ret
    }
}

impl Input {
    pub fn to_num(self) -> u8 {
        match self {
            Input::Number(nr) => nr,
            Input::Random => random(),
        }
    }
}
