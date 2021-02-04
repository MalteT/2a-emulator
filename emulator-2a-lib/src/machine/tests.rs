use proptest::prelude::*;

use std::fs::read_to_string;

use super::*;
use crate::{
    compiler::Translator,
    parser::AsmParser,
    runner::{RunExpectationsBuilder, RunnerConfigBuilder},
};

macro_rules! run {
    {
        path = $path:literal;
        config = $with:expr;
        expect = $expectation:expr;
    } => {
        let program = read_to_string($path).expect("Failed to read program");
        let expectation = $expectation.build().expect("Failed to create expectations");
        let runner = $with.with_program(&program).build().expect("RunConfig creation failed");
        let result = runner.run().expect("Failed to parse program");
        if let Err(why) = expectation.verify(&result) {
            panic!(" -> Test failed: {}", why)
        }
    }
}

macro_rules! compile {
    { $program:literal } => {
        {
            let asm = AsmParser::parse($program).expect("Failed to parse program");
            Translator::compile(&asm)
        }
    }
}

macro_rules! load {
    { $program:literal } => {
        {
            let mut machine = Machine::new(MachineConfig::default());
            machine.load(compile!($program));
            machine
        }
    }
}

proptest! {
    #[test]
    fn step_mode_is_never_reset(starting_step_mode: StepMode) {
        let mut machine = Machine::new(MachineConfig::default());
        machine.step_mode = starting_step_mode;
        machine.cpu_reset();
        assert_eq!(machine.step_mode(), starting_step_mode);
        machine.master_reset();
        assert_eq!(machine.step_mode(), starting_step_mode);
    }

    #[test]
    fn key_edge_interrupts_work(interrupt_cycle in 50_usize..=100) {
        // To make sure, that the interrupt firing works independent of the cycle, fuzz!
        // The setup takes about 40 cycles, fire the interrupt in
        // the upcoming 50-100 cycles and leave 50 cycles for the ISR
        run! {
            path = "../testing/programs/12-simple-key-interrupt-check.asm";
            config = RunnerConfigBuilder::default()
                .with_max_cycles(150)
                .with_interrupts([interrupt_cycle]);
            expect = RunExpectationsBuilder::default()
                .expect_state(State::ErrorStopped)
                .expect_output_ff(1);
        }
    }

    #[test]
    fn key_interrupt_is_set_in_misr_correctly(interrupt_cycle in 500_usize..=550) {
        run! {
            path = "../testing/programs/13-misr-testing-during-key-interrupts.asm";
            config = RunnerConfigBuilder::default()
                .with_max_cycles(1000)
                .with_interrupts([interrupt_cycle]);
            expect = RunExpectationsBuilder::default()
                .expect_output_ff(0);
        }
    }
}

// XXX: Not supported yet
// #[test]
// fn org_assembly_instruction_works() {
//     let machine = load! {
//         r#"#! mrasm
//             .ORG 1
//             STOP
//             STOP
//             .ORG 2
//             .DB 0x42
//         "#
//     };
//     // The first byte has to be zero, since we start an ORG 1
//     assert_eq!(machine.bus().read(0), 0);
//     // The next byte is the STOP instruction
//     assert_eq!(machine.bus().read(1), 1);
//     // The last step was overwritten by the .DB
//     assert_eq!(machine.bus().read(2), 0x42);
//     assert_eq!(machine.bus().read(3), 0);
// }

#[test]
fn org_assembly_instruction_works() {
    let machine = load! {
        r#"#! mrasm
            .ORG 1
            STOP
            STOP
            .ORG 4
            .DB 0x42
        "#
    };
    // The first byte has to be zero, since we start an ORG 1
    assert_eq!(machine.bus().read(0), 0);
    // The next two byte are the STOP instructions
    assert_eq!(machine.bus().read(1), 1);
    assert_eq!(machine.bus().read(2), 1);
    // Skip one
    assert_eq!(machine.bus().read(3), 0);
    assert_eq!(machine.bus().read(4), 0x42);
    assert_eq!(machine.bus().read(5), 0);
}

#[test]
fn byte_assembly_instruction_works() {
    let machine = load! {
        r#"#! mrasm
            .BYTE 2
            STOP
            .BYTE 2
            .DB 0x57
            .BYTE 1
            STOP
        "#
    };
    assert_eq!(machine.bus().read(0), 0);
    assert_eq!(machine.bus().read(1), 0);
    assert_eq!(machine.bus().read(2), 1);
    assert_eq!(machine.bus().read(3), 0);
    assert_eq!(machine.bus().read(4), 0);
    assert_eq!(machine.bus().read(5), 0x57);
    assert_eq!(machine.bus().read(6), 0);
    assert_eq!(machine.bus().read(7), 1);
    assert_eq!(machine.bus().read(8), 0);
}

#[test]
fn ram_is_reset_on_program_load() {
    let mut machine = Machine::new(MachineConfig::default());
    // This takes up some bytes
    let program_1 = r#"#! mrasm
        LOOP:
            INC R0
            JR LOOP
        "#;
    // This will only be one byte, thus overwriting only the INC R0
    let program_2 = r#"#! mrasm
            NOP
        "#;
    let asm = AsmParser::parse(program_1).unwrap();
    let bytes = Translator::compile(&asm);
    machine.load(bytes);
    // Second byte is non-zero
    assert_ne!(machine.bus().read(1), 0);
    let asm = AsmParser::parse(program_2).unwrap();
    let bytes = Translator::compile(&asm);
    machine.load(bytes);
    // Second byte should be zero again
    assert_eq!(machine.bus().read(1), 0);
}

#[test]
fn stacksize_default_limit_works() {
    run! {
        path = "../testing/programs/01-stacksize-16.asm";
        config = RunnerConfigBuilder::default().with_max_cycles(1_000);
        expect = RunExpectationsBuilder::default()
            .expect_state(State::ErrorStopped)
            .expect_output_ff(16);
    }
}

#[test]
fn stacksize_limit_no_set_works() {
    run! {
        path = "../testing/programs/06-stacksize-noset.asm";
        config = RunnerConfigBuilder::default().with_max_cycles(1_000);
        expect = RunExpectationsBuilder::default()
            .expect_state(State::ErrorStopped)
            .expect_output_ff(16);
    }
}

#[test]
fn stacksize_limit_32_works() {
    run! {
        path = "../testing/programs/02-stacksize-32.asm";
        config = RunnerConfigBuilder::default().with_max_cycles(1_000);
        expect = RunExpectationsBuilder::default()
            .expect_state(State::ErrorStopped)
            .expect_output_ff(32);
    }
}

#[test]
fn stacksize_limit_48_works() {
    run! {
        path = "../testing/programs/03-stacksize-48.asm";
        config = RunnerConfigBuilder::default().with_max_cycles(10_000);
        expect = RunExpectationsBuilder::default()
            .expect_state(State::ErrorStopped)
            .expect_output_ff(48);
    }
}

#[test]
fn stacksize_limit_64_works() {
    run! {
        path = "../testing/programs/04-stacksize-64.asm";
        config = RunnerConfigBuilder::default().with_max_cycles(10_000);
        expect = RunExpectationsBuilder::default()
            .expect_state(State::ErrorStopped)
            .expect_output_ff(64);
    }
}

#[test]
fn stacksize_limit_0_works() {
    // The limit is rather arbitrary here
    run! {
        path = "../testing/programs/05-stacksize-0.asm";
        config = RunnerConfigBuilder::default().with_max_cycles(10_000);
        expect = RunExpectationsBuilder::default()
            .expect_state(State::ErrorStopped)
            .expect_output_ff(229);
    }
}

#[test]
fn test_program_loading() {
    let mut machine = Machine::new(MachineConfig::default());
    let prog = &["#! mrasm", ".DB 42"].join("\n");
    let parsed = AsmParser::parse(prog).expect("Parsing failed");
    let compiled = Translator::compile(&parsed);
    machine.load(compiled);
    assert_eq!(machine.bus().memory()[0], 42);
}

#[test]
fn test_stackpointer_when_loading() {
    let mut machine = Machine::new(MachineConfig::default());
    let mut load_verify = |program: &str, ss: Stacksize| {
        let asm = AsmParser::parse(program).expect("Parsing failed");
        let bytecode = Translator::compile(&asm);
        machine.load(bytecode);
        assert_eq!(machine.stacksize(), ss)
    };
    let program_asm_default = &["#! mrasm"].join("\n");
    load_verify(program_asm_default, Stacksize::_16);

    let program_asm_0 = &["#! mrasm", "*STACKSIZE 0"].join("\n");
    load_verify(program_asm_0, Stacksize::_0);

    let program_asm_16 = &["#! mrasm", "*STACKSIZE 16"].join("\n");
    load_verify(program_asm_16, Stacksize::_16);

    let program_asm_64 = &["#! mrasm", "*STACKSIZE 64"].join("\n");
    load_verify(program_asm_64, Stacksize::_64);

    let program_asm_no_set = &["#! mrasm", "*STACKSIZE NOSET"].join("\n");
    load_verify(program_asm_no_set, Stacksize::_64);
}

#[test]
fn misr_is_set_correctly_by_key_interrupt() {
    let mut machine = Machine::new(MachineConfig::default());
    machine.raw_mut().bus_mut().write(0xF9, 0b0000_0001);
    let misr = machine.bus().read(0xF9);
    assert_eq!(misr & 0b0000_0001, 0b0000_0000);
    machine.trigger_key_interrupt();
    let misr = machine.bus().read(0xF9);
    assert_eq!(misr & 0b0000_0001, 0b0000_0001);
}
