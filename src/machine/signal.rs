use super::{Instruction, MP28BitWord};

/// The collection of all relevant signals with
/// simplified methods that all return [`bool`]s to easily
/// work with the current internal state.
pub struct Signal {
    word: MP28BitWord,
    inst: Instruction,
    carry_flag: Option<bool>,
    zero_flag: Option<bool>,
    negative_flag: Option<bool>,
    interrupt_enable_flag: Option<bool>,
    carry_out: Option<bool>,
    zero_out: Option<bool>,
    negative_out: Option<bool>,
    edge_interrupt: Option<bool>,
    level_interrupt: Option<bool>,
}

impl Signal {
    /// Create a new Signal struct from the current word and byte instruction.
    pub fn new(word: &MP28BitWord, inst: &Instruction) -> Signal {
        let word = word.clone();
        let inst = inst.clone();
        let carry_flag = None;
        let zero_flag = None;
        let negative_flag = None;
        let interrupt_enable_flag = None;
        let carry_out = None;
        let zero_out = None;
        let negative_out = None;
        let edge_interrupt = None;
        let level_interrupt = None;
        Signal {
            word,
            inst,
            carry_flag,
            zero_flag,
            negative_flag,
            carry_out,
            zero_out,
            negative_out,
            interrupt_enable_flag,
            edge_interrupt,
            level_interrupt,
        }
    }
    pub fn set_current_instruction(&mut self, inst: &Instruction) {
        self.inst = inst.clone();
    }
    pub fn set_cf(&mut self, carry_flag: bool) {
        self.carry_flag = Some(carry_flag)
    }
    pub fn set_zf(&mut self, zero_flag: bool) {
        self.zero_flag = Some(zero_flag)
    }
    pub fn set_nf(&mut self, negative_flag: bool) {
        self.negative_flag = Some(negative_flag)
    }
    pub fn set_ief(&mut self, interrupt_enable_flag: bool) {
        self.interrupt_enable_flag = Some(interrupt_enable_flag)
    }
    pub fn set_co(&mut self, carry_out: bool) {
        self.carry_out = Some(carry_out)
    }
    pub fn set_zo(&mut self, zero_out: bool) {
        self.zero_out = Some(zero_out)
    }
    pub fn set_no(&mut self, negative_out: bool) {
        self.negative_out = Some(negative_out)
    }
    pub fn set_edge_int(&mut self, edge_interrupt: bool) {
        self.edge_interrupt = Some(edge_interrupt)
    }
    pub fn set_level_int(&mut self, level_interrupt: bool) {
        self.level_interrupt = Some(level_interrupt)
    }
    pub fn a8(&self) -> bool {
        self.inst.contains(Instruction::A8)
    }
    pub fn a7(&self) -> bool {
        self.inst.contains(Instruction::A7)
    }
    pub fn a6(&self) -> bool {
        self.inst.contains(Instruction::A6)
    }
    pub fn a5(&self) -> bool {
        self.inst.contains(Instruction::A5)
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
    pub fn na4(&self) -> bool {
        self.word.contains(MP28BitWord::NA4)
    }
    pub fn na3(&self) -> bool {
        self.word.contains(MP28BitWord::NA3)
    }
    pub fn na2(&self) -> bool {
        self.word.contains(MP28BitWord::NA2)
    }
    pub fn na1(&self) -> bool {
        self.word.contains(MP28BitWord::NA1)
    }
    pub fn na0(&self) -> bool {
        self.word.contains(MP28BitWord::NA0)
    }
    pub fn mac0(&self) -> bool {
        self.word.contains(MP28BitWord::MAC0)
    }
    pub fn mac1(&self) -> bool {
        self.word.contains(MP28BitWord::MAC1)
    }
    pub fn mac2(&self) -> bool {
        self.word.contains(MP28BitWord::MAC2)
    }
    #[allow(dead_code)]
    pub fn mac3(&self) -> bool {
        self.word.contains(MP28BitWord::MAC3)
    }
    pub fn busen(&self) -> bool {
        self.word.contains(MP28BitWord::BUSEN)
    }
    pub fn buswr(&self) -> bool {
        self.word.contains(MP28BitWord::BUSWR)
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
    pub fn maluia(&self) -> bool {
        self.word.contains(MP28BitWord::MALUIA)
    }
    pub fn maluib(&self) -> bool {
        self.word.contains(MP28BitWord::MALUIB)
    }
    pub fn malus3(&self) -> bool {
        self.word.contains(MP28BitWord::MALUS3)
    }
    pub fn malus2(&self) -> bool {
        self.word.contains(MP28BitWord::MALUS2)
    }
    pub fn malus1(&self) -> bool {
        self.word.contains(MP28BitWord::MALUS1)
    }
    pub fn malus0(&self) -> bool {
        self.word.contains(MP28BitWord::MALUS0)
    }
    pub fn mrgwe(&self) -> bool {
        self.word.contains(MP28BitWord::MRGWE)
    }
    pub fn mrgws(&self) -> bool {
        self.word.contains(MP28BitWord::MRGWS)
    }
    pub fn mchflg(&self) -> bool {
        self.word.contains(MP28BitWord::MCHFLG)
    }
    pub fn cf(&self) -> bool {
        *self
            .carry_flag
            .as_ref()
            .expect("BUG: Carry flag not added to signals yet")
    }
    pub fn zf(&self) -> bool {
        *self
            .zero_flag
            .as_ref()
            .expect("BUG: Zero flag not added to signals yet")
    }
    pub fn nf(&self) -> bool {
        *self
            .negative_flag
            .as_ref()
            .expect("BUG: Negative flag not added to signals yet")
    }
    pub fn ief(&self) -> bool {
        *self
            .interrupt_enable_flag
            .as_ref()
            .expect("BUG: Interrupt enable flag not added to signals yet")
    }
    pub fn co(&self) -> bool {
        *self
            .carry_out
            .as_ref()
            .expect("BUG: Carry out not added to signals yet")
    }
    pub fn zo(&self) -> bool {
        *self
            .zero_out
            .as_ref()
            .expect("BUG: Zero out not added to signals yet")
    }
    pub fn no(&self) -> bool {
        *self
            .negative_out
            .as_ref()
            .expect("BUG: Negative out not added to signals yet")
    }
    pub fn level_int(&self) -> bool {
        *self
            .level_interrupt
            .as_ref()
            .expect("BUG: Level interrupt not added to signals yet")
    }
    pub fn edge_int(&self) -> bool {
        *self
            .edge_interrupt
            .as_ref()
            .expect("BUG: Edge interrupt not added to signals yet")
    }
}
