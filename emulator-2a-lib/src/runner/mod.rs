use derive_builder::Builder;
use thiserror::Error;

use std::{
    marker::PhantomData,
    time::{Duration, Instant},
};

use crate::{
    compiler::Translator,
    machine::{Machine, MachineConfig, State},
    parser::{AsmParser, ParserError},
};

#[derive(Debug, Builder, Clone, PartialEq)]
#[builder(setter(prefix = "with"))]
pub struct RunnerConfig<'a> {
    /// Maximum number of cycles to emulate.
    pub max_cycles: usize,
    /// Configuration for the machine.
    /// The machine will be initialized with this configuration.
    #[builder(default)]
    pub machine_config: MachineConfig,
    /// Program to run on the machine.
    pub program: &'a str,
    /// A list of cycles at which to trigger a key edge interrupt.
    #[builder(default, setter(into))]
    pub interrupts: Vec<usize>,
    /// A list of cycles at which to trigger a cpu reset.
    #[builder(default, setter(into))]
    pub resets: Vec<usize>,
    /// Prevent the manual creation of this struct for the purpose of extension
    #[builder(setter(skip), default)]
    _phantom: PhantomData<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunResults<'a> {
    /// Machine in the state after the last cycle.
    pub machine: Machine,
    /// Number of cycles that were emulated.
    /// This might differ from the maximum
    /// number of cycles if the machine halted.
    pub emulated_cycles: usize,
    /// The time the emulation took.
    pub time_taken: Duration,
    /// Reference to the configuration that was used to
    /// generate this result.
    pub config: &'a RunnerConfig<'a>,
    /// Prevent the manual creation of this struct for the purpose of extension
    _phantom: PhantomData<u8>,
}

#[derive(Debug, Error)]
pub enum VerificationError {
    #[error("State == {found:?} != {expected:?}")]
    StateMismatch { expected: State, found: State },
    #[error("Output Register FE == {found} != {expected}")]
    OutputFeMismatch { expected: u8, found: u8 },
    #[error("Output Register FF == {found} != {expected}")]
    OutputFfMismatch { expected: u8, found: u8 },
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Builder)]
#[builder(setter(prefix = "expect", strip_option), default)]
pub struct RunExpectations {
    /// The state that the program should have after execution.
    state: Option<State>,
    /// Expected output register FE
    output_fe: Option<u8>,
    /// Expected output register FF
    output_ff: Option<u8>,
}

impl<'a> RunnerConfig<'a> {
    /// Execute the runner.
    ///
    /// This executes the runner and checks all verifications.
    pub fn run(&self) -> Result<RunResults, ParserError> {
        // Prepare the machine
        let parsed = AsmParser::parse(self.program)?;
        let bytecode = Translator::compile(&parsed);
        let mut machine = Machine::new_with_program(self.machine_config.clone(), bytecode);
        // Initialize variables
        let before_emulation = Instant::now();
        let mut emulated_cycles = 0;
        // RUN!
        while emulated_cycles < self.max_cycles {
            // Prerequisites for the cycle
            if self.interrupts.contains(&emulated_cycles) {
                machine.trigger_key_interrupt();
            }
            if self.resets.contains(&emulated_cycles) {
                machine.cpu_reset();
            }
            // Trigger the next cycle
            machine.trigger_key_clock();
            emulated_cycles += 1;
            // Bail if possible
            if machine.state() != State::Running {
                break;
            }
        }
        // Assemble results
        Ok(RunResults {
            config: self,
            time_taken: before_emulation.elapsed(),
            emulated_cycles,
            machine,
            _phantom: PhantomData,
        })
    }
}

impl RunExpectations {
    pub fn verify(&self, result: &RunResults) -> Result<(), VerificationError> {
        if self.state.is_some() && self.state != Some(result.machine.state()) {
            Err(VerificationError::StateMismatch {
                expected: self.state.unwrap(),
                found: result.machine.state(),
            })
        } else if self.output_fe.is_some()
            && self.output_fe != Some(result.machine.bus().output_fe())
        {
            Err(VerificationError::OutputFeMismatch {
                expected: self.output_fe.unwrap(),
                found: result.machine.bus().output_fe(),
            })
        } else if self.output_ff.is_some()
            && self.output_ff != Some(result.machine.bus().output_ff())
        {
            Err(VerificationError::OutputFfMismatch {
                expected: self.output_ff.unwrap(),
                found: result.machine.bus().output_ff(),
            })
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_runner_functions_work() {
        let program = r#"#! mrasm
                INC R0
                MOV (0xFE), R0
            LOOP:
                JR LOOP
        "#;
        let config = RunnerConfigBuilder::default()
            .with_max_cycles(10_000)
            .with_program(program)
            .build()
            .unwrap();
        let res = config.run().expect("Parsing failed");
        let expectations = RunExpectationsBuilder::default()
            .expect_state(State::Running)
            .expect_output_fe(1)
            .build()
            .unwrap();
        expectations.verify(&res).expect("Verification failed");
    }

    #[test]
    fn runner_resets_work_correctly() {
        let program = r#"#! mrasm
            LOOP:
                INC R0
                ST (0xFF), R0
                JR LOOP
        "#;
        let config = RunnerConfigBuilder::default()
            .with_max_cycles(17) // One iteration
            .with_program(program)
            .build()
            .unwrap();
        let res = config.run().expect("Parsing failed");
        let expectations = RunExpectationsBuilder::default()
            .expect_state(State::Running)
            .expect_output_ff(1)
            .build()
            .unwrap();
        expectations.verify(&res).expect("Verification failed");
        let config = RunnerConfigBuilder::default()
            .with_max_cycles(34) // Two iterations
            .with_program(program)
            .build()
            .unwrap();
        let res = config.run().expect("Parsing failed");
        let expectations = RunExpectationsBuilder::default()
            .expect_state(State::Running)
            .expect_output_ff(2)
            .build()
            .unwrap();
        expectations.verify(&res).expect("Verification failed");
        // Now for the test
        let config = RunnerConfigBuilder::default()
            .with_max_cycles(10_000) // Forever!
            .with_resets([9_966]) // But we reset the machine at the right moment
            .with_program(program)
            .build()
            .unwrap();
        let res = config.run().expect("Parsing failed");
        let expectations = RunExpectationsBuilder::default()
            .expect_state(State::Running)
            .expect_output_ff(2)
            .build()
            .unwrap();
        expectations.verify(&res).expect("Verification failed");
    }

    #[test]
    fn runner_interrupts_work_correctly() {
        let program = r#"#! mrasm
                JR MAIN         ; 4-5 cycles
                JR ISR
            MAIN:
                LDSP 0xEF       ; 5-9 cycles
                BITS (0xF9), 1  ; 8-17 cycles
                EI              ; 4 cycles (total: 21-35 cycles setup)
            LOOP:
                INC R0          ; 3 cycles
                JR LOOP         ; 4-5 cycles
            ISR:
                ST (0xFF), R0
                STOP
        "#;
        let config = RunnerConfigBuilder::default()
            .with_max_cycles(10_000)
            .with_program(program)
            .with_interrupts([5_000])
            .build()
            .unwrap();
        let res = config.run().expect("Parsing failed");
        let expectations = RunExpectationsBuilder::default()
            .expect_state(State::Stopped)
            .expect_output_ff(110) // This is just a guess..
            .build()
            .unwrap();
        expectations.verify(&res).expect("Verification failed");
    }
}
