use enum_primitive::FromPrimitive;

use crate::machine::Instruction;
use crate::{AluSelect, Flags, Word};

/// The collection of all relevant internal signals with
/// simplified methods that all return [`bool`]s to easily
/// work with the current internal state.
pub struct Signal<'w, 'i> {
    word: &'w Word,
    instruction: &'i Instruction,
    flags: Flags,
    carry_out: bool,
    zero_out: bool,
    negative_out: bool,
}

impl<'w, 'i> Signal<'w, 'i> {
    /// Create a new Signal struct from the current word and byte instruction.
    pub const fn new() -> Signal<'static, 'static> {
        Signal {
            word: &Word::empty(),
            instruction: &Instruction::empty(),
            flags: Flags::empty(),
            carry_out: false,
            zero_out: false,
            negative_out: false,
        }
    }
    pub const fn set_instruction<'i1>(self, instruction: &'i1 Instruction) -> Signal<'w, 'i1> {
        Signal {
            word: self.word,
            instruction,
            flags: self.flags,
            carry_out: self.carry_out,
            zero_out: self.zero_out,
            negative_out: self.negative_out,
        }
    }
    pub const fn set_flags(self, flags: Flags) -> Self {
        Signal { flags, ..self }
    }
    pub const fn set_carry_out(self, carry_out: bool) -> Self {
        Signal { carry_out, ..self }
    }
    pub const fn set_zero_out(self, zero_out: bool) -> Self {
        Signal { zero_out, ..self }
    }
    pub const fn set_negative_out(self, negative_out: bool) -> Self {
        Signal {
            negative_out,
            ..self
        }
    }
    pub const fn a8(&self) -> bool {
        self.instruction.contains(Instruction::A8)
    }
    pub const fn a7(&self) -> bool {
        self.instruction.contains(Instruction::A7)
    }
    pub const fn a6(&self) -> bool {
        self.instruction.contains(Instruction::A6)
    }
    pub const fn a5(&self) -> bool {
        self.instruction.contains(Instruction::A5)
    }
    pub const fn op00(&self) -> bool {
        self.instruction.contains(Instruction::OP00)
    }
    pub const fn op01(&self) -> bool {
        self.instruction.contains(Instruction::OP01)
    }
    pub const fn op10(&self) -> bool {
        self.instruction.contains(Instruction::OP10)
    }
    pub const fn op11(&self) -> bool {
        self.instruction.contains(Instruction::OP11)
    }
    pub const fn na4(&self) -> bool {
        self.word.contains(Word::NA4)
    }
    pub const fn na3(&self) -> bool {
        self.word.contains(Word::NA3)
    }
    pub const fn na2(&self) -> bool {
        self.word.contains(Word::NA2)
    }
    pub const fn na1(&self) -> bool {
        self.word.contains(Word::NA1)
    }
    pub const fn na0(&self) -> bool {
        self.word.contains(Word::NA0)
    }
    pub const fn mac0(&self) -> bool {
        self.word.contains(Word::MAC0)
    }
    pub const fn mac1(&self) -> bool {
        self.word.contains(Word::MAC1)
    }
    pub const fn mac2(&self) -> bool {
        self.word.contains(Word::MAC2)
    }
    pub const fn mac3(&self) -> bool {
        self.word.contains(Word::MAC3)
    }
    pub const fn busen(&self) -> bool {
        self.word.contains(Word::BUSEN)
    }
    pub const fn buswr(&self) -> bool {
        self.word.contains(Word::BUSWR)
    }
    pub const fn mrgaa0(&self) -> bool {
        self.word.contains(Word::MRGAA0)
    }
    pub const fn mrgaa1(&self) -> bool {
        self.word.contains(Word::MRGAA1)
    }
    pub const fn mrgaa2(&self) -> bool {
        self.word.contains(Word::MRGAA2)
    }
    pub const fn mrgaa3(&self) -> bool {
        self.word.contains(Word::MRGAA3)
    }
    pub const fn mrgab0(&self) -> bool {
        self.word.contains(Word::MRGAB0)
    }
    pub const fn mrgab1(&self) -> bool {
        self.word.contains(Word::MRGAB1)
    }
    pub const fn mrgab2(&self) -> bool {
        self.word.contains(Word::MRGAB2)
    }
    pub const fn mrgab3(&self) -> bool {
        self.word.contains(Word::MRGAB3)
    }
    pub const fn maluia(&self) -> bool {
        self.word.contains(Word::MALUIA)
    }
    pub const fn maluib(&self) -> bool {
        self.word.contains(Word::MALUIB)
    }
    pub const fn malus3(&self) -> bool {
        self.word.contains(Word::MALUS3)
    }
    pub const fn malus2(&self) -> bool {
        self.word.contains(Word::MALUS2)
    }
    pub const fn malus1(&self) -> bool {
        self.word.contains(Word::MALUS1)
    }
    pub const fn malus0(&self) -> bool {
        self.word.contains(Word::MALUS0)
    }
    pub const fn mrgwe(&self) -> bool {
        self.word.contains(Word::MRGWE)
    }
    pub const fn mrgws(&self) -> bool {
        self.word.contains(Word::MRGWS)
    }
    pub const fn mchflg(&self) -> bool {
        self.word.contains(Word::MCHFLG)
    }
    pub const fn carry_flag(&self) -> bool {
        self.flags.contains(Flags::CARRY_FLAG)
    }
    pub const fn zero_flag(&self) -> bool {
        self.flags.contains(Flags::ZERO_FLAG)
    }
    pub const fn negative_flag(&self) -> bool {
        self.flags.contains(Flags::NEGATIVE_FLAG)
    }
    pub const fn interrupt_enable_flag(&self) -> bool {
        self.flags.contains(Flags::INTERRUPT_ENABLE_FLAG)
    }
    pub const fn carry_out(&self) -> bool {
        self.carry_out
    }
    pub const fn zero_out(&self) -> bool {
        self.zero_out
    }
    pub const fn negative_out(&self) -> bool {
        self.negative_out
    }
    /// Get the function to execute inside the ALU.
    pub fn alu_select(&self) -> AluSelect {
        let malus3 = self.malus3() as u8;
        let malus2 = self.malus2() as u8;
        let malus1 = self.malus1() as u8;
        let malus0 = self.malus0() as u8;
        let select = (malus3 << 3) | (malus2 << 2) | (malus1 << 1) | malus0;
        AluSelect::from_u8(select).expect("infallible")
    }
}
