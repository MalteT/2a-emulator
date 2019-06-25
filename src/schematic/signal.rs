use super::{Instruction, MP28BitWord};

/// The collection of all relevant signals with
/// simplified methods that all return [`bool`]s to easily
/// work with the current internal state.
pub struct Signal<'a> {
    word: &'a MP28BitWord,
    inst: &'a Instruction,
}

impl<'a> Signal<'a> {
    /// Create a new Signal struct from the current word and byte instruction.
    pub fn new(word: &'a MP28BitWord, inst: &'a Instruction) -> Signal<'a> {
        Signal { word, inst }
    }
    pub fn op00(&self) -> bool {
        self.inst.contains(Instruction::OP00)
    }
    pub fn op01(&self) -> bool {
        self.inst.contains(Instruction::OP01)
    }
    pub fn op10(&self) -> bool {
        self.inst.contains(Instruction::OP10)
    }
    pub fn op11(&self) -> bool {
        self.inst.contains(Instruction::OP11)
    }
    pub fn mrgaa0(&self) -> bool {
        self.word.contains(MP28BitWord::MRGAA0)
    }
    pub fn mrgaa1(&self) -> bool {
        self.word.contains(MP28BitWord::MRGAA1)
    }
    pub fn mrgaa2(&self) -> bool {
        self.word.contains(MP28BitWord::MRGAA2)
    }
    pub fn mrgaa3(&self) -> bool {
        self.word.contains(MP28BitWord::MRGAA3)
    }
    pub fn mrgab0(&self) -> bool {
        self.word.contains(MP28BitWord::MRGAB0)
    }
    pub fn mrgab1(&self) -> bool {
        self.word.contains(MP28BitWord::MRGAB1)
    }
    pub fn mrgab2(&self) -> bool {
        self.word.contains(MP28BitWord::MRGAB2)
    }
    pub fn mrgab3(&self) -> bool {
        self.word.contains(MP28BitWord::MRGAB3)
    }
}
