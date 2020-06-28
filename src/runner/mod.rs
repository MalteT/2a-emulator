use crate::machine::Machine;

pub struct Runner {
    machine: Machine,
}

pub struct RunOptions {}

pub struct RunOptionsBuilder;

impl RunOptions {
    pub const fn default() -> RunOptionsBuilder {
        RunOptionsBuilder
    }
}

pub struct RunResults {}
