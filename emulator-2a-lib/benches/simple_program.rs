use criterion::{black_box, criterion_group, criterion_main, Criterion};
use emulator_2a_lib::{
    compiler::Translator,
    machine::{Machine, MachineConfig, State},
    parser::AsmParser,
};

mod perf;

const PROGRAM: &str = r#"#! mrasm
    INC R0
    MOV (0xFF), R0
"#;

pub fn simple_move(c: &mut Criterion) {
    let mut machine = Machine::new(MachineConfig::default());
    machine.set_step_mode(emulator_2a_lib::machine::StepMode::Assembly);
    let parsed = AsmParser::parse(PROGRAM).expect("Program not parseable");
    let bytecode = Translator::compile(&parsed);
    machine.load(bytecode);
    c.bench_function("run simple program", |b| {
        b.iter(|| run_program(black_box(PROGRAM)))
    });
}

pub fn run_program(program: &str) {
    let mut machine = Machine::new(MachineConfig::default());
    let parsed = AsmParser::parse(program).expect("Program not parseable");
    let bytecode = Translator::compile(&parsed);
    machine.load(bytecode);
    while machine.state() == State::Running {
        machine.trigger_key_clock()
    }
}

criterion_main!(benches);

criterion_group! {
    name = benches;
    // This can be any expression that returns a `Criterion` object.
    config = Criterion::default().with_profiler(perf::FlamegraphProfiler::new(100));
    targets = simple_move
}
