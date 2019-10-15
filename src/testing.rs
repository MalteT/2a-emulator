use log::{info, trace};
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use rand::random;

use std::fs::read_to_string;
use std::path::PathBuf;

use crate::error::Error;
use crate::helpers;
use crate::machine::Machine;

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
    Halt,
    NoHalt,
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
    pub fn execute_against<P: Into<PathBuf>>(&self, path: P) -> Result<(), Error> {
        let path: PathBuf = path.into();
        for test in &self.tests {
            trace!("Executing test {:?} for {:?}", test.name, path);
            test.execute_against(&path)?;
        }
        Ok(())
    }
}

impl Test {
    /// Execute the test.
    pub fn execute_against<P: Into<PathBuf>>(&self, path: P) -> Result<(), Error> {
        let path: PathBuf = path.into();
        let program = helpers::read_asm_file(&path)?;
        let mut machine = Machine::new(Some(&program));
        // Prepare
        let to_num = |inp| match inp {
            Input::Number(nr) => nr,
            Input::Random => random(),
        };
        let mut random_reset = false;
        let mut random_interrupt = false;
        let mut interrupt = false;
        for setting in &self.settings {
            match setting {
                Setting::InputFc(nr) => machine.input_fc(to_num(*nr)),
                Setting::InputFd(nr) => machine.input_fd(to_num(*nr)),
                Setting::InputFe(nr) => machine.input_fe(to_num(*nr)),
                Setting::InputFf(nr) => machine.input_ff(to_num(*nr)),
                Setting::RandomInput => {
                    machine.input_fc(random());
                    machine.input_fd(random());
                    machine.input_fe(random());
                    machine.input_ff(random());
                }
                Setting::RandomReset => random_reset = true,
                Setting::RandomInterrupt => random_interrupt = true,
                Setting::Interrupt => interrupt = true,
            }
        }

        // Execute
        for i in 0..self.ticks {
            if random_reset && 0.95 < random() {
                machine.reset();
                trace!("Test: Randomly reseted machine");
            }
            if random_interrupt && 0.95 < random() {
                machine.key_edge_int();
                trace!("Test: Randomly set edge interrupt on machine");
            }
            if interrupt && i > self.ticks / 2 {
                interrupt = false;
                machine.key_edge_int();
                trace!("Test: Interrupted the machine once");
            }
            machine.clk()
        }

        // Verify
        trace!("Verifying test {:?} for {:?}", self.name, path);
        for expectation in &self.expectations {
            match expectation {
                Expectation::Halt => {
                    if !(machine.is_stopped() || machine.is_error_stopped()) {
                        return Err(Error::TestFailed("Machine did not halt!".into()));
                    }
                }
                Expectation::NoHalt => {
                    if machine.is_stopped() || machine.is_error_stopped() {
                        return Err(Error::TestFailed("Machine halted!".into()));
                    }
                }
                Expectation::OutputFe(nr) => {
                    if machine.output_fe() != *nr {
                        return Err(Error::TestFailed(format!(
                            "Different output on FE: {} != {}",
                            nr,
                            machine.output_fe()
                        )));
                    }
                }
                Expectation::OutputFf(nr) => {
                    if machine.output_ff() != *nr {
                        return Err(Error::TestFailed(format!(
                            "Different output on FF: {} != {}",
                            nr,
                            machine.output_ff()
                        )));
                    }
                }
            };
        }
        info!("Test {:?} for {:?} was successful", self.name, path);
        Ok(())
    }
    /// Parse a test from the given Pest Pair.
    fn parse(pair: Pair<Rule>) -> Self {
        let mut name = "".into();
        let mut ticks = 10_000;
        let mut settings = vec![];
        let mut expectations = vec![Expectation::NoHalt];

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
                Rule::halt => Expectation::Halt,
                Rule::no_halt => Expectation::NoHalt,
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
