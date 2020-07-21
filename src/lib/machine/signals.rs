use enum_primitive::FromPrimitive;

use super::Machine;
use crate::{AluSelect, Flags, Instruction, Interrupt, RegisterNumber, Word};

/// The collection of all relevant internal signals with
/// simplified methods that all return [`bool`]s to easily
/// work with the current internal state.
pub struct Signals<'a> {
    word: &'a Word,
    instruction: &'a Instruction,
    interrupt_flipflop_1: Option<&'a Interrupt>,
    level_interrupt: Option<&'a Interrupt>,
    flags: Flags,
    carry_out: bool,
    zero_out: bool,
    negative_out: bool,
}

impl<'a> Signals<'a> {
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
    /// Get the value of the interrupt flip flop IFF1.
    pub fn interrupt_flipflop_1(&self) -> bool {
        self.interrupt_flipflop_1.is_some()
    }
    pub fn level_interrupt(&self) -> bool {
        self.level_interrupt.is_some()
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
    /// Get the [`RegisterNumber`] of the register that is selected by
    /// the select inputs AA2..AA0.
    pub fn selected_register_a(&self) -> RegisterNumber {
        let (aa2, aa1, aa0) = if self.mrgaa3() {
            (false, self.op01(), self.op00())
        } else {
            (self.mrgaa2(), self.mrgaa1(), self.mrgaa0())
        };
        let addr = ((aa2 as u8) << 2) + ((aa1 as u8) << 1) + (aa0 as u8);
        RegisterNumber::from_u8(addr).expect("Infallible")
    }
    /// Get the [`RegisterNumber`] of the register that is selected by
    /// the select inputs AB2..AB0.
    pub fn selected_register_b(&self) -> RegisterNumber {
        let (ab2, ab1, ab0) = if self.mrgaa3() {
            (false, self.op01(), self.op00())
        } else {
            (self.mrgaa2(), self.mrgaa1(), self.mrgaa0())
        };
        let addr = ((ab2 as u8) << 2) + ((ab1 as u8) << 1) + (ab0 as u8);
        RegisterNumber::from_u8(addr).expect("Infallible")
    }
    /// Get the constant that can be used as the alu input b.
    /// The lower 3 bit contain MRGAB2..MRGAB0, the upper 5 bit are set
    /// to MRGAB3.
    pub const fn alu_input_b_constant(&self) -> u8 {
        (0b1111_1000 * self.mrgab3() as u8) + (self.mrgab2() as u8)
            << 2 + (self.mrgab1() as u8)
            << 1 + (self.mrgab0() as u8)
    }
    /// Get the [`RegisterNumber`] that will is selected for writing.
    pub fn selected_register_for_writing(&self) -> RegisterNumber {
        if self.mrgws() {
            self.selected_register_b()
        } else {
            self.selected_register_a()
        }
    }
    /// Get the next address of the [`MicroprogramRam`].
    pub fn next_microprogram_address(&self) -> usize {
        (self.a8() as usize)
            << 8 + (self.a7() as usize)
            << 7 + (self.a6() as usize)
            << 6 + (self.a5() as usize)
            << 5 + (self.na4() as usize)
            << 4 + (self.na3() as usize)
            << 3 + (self.na2() as usize)
            << 2 + (self.am4() as usize)
            << 1 + (self.am3() as usize)
    }
    /// Get the output of the address multiplexer AM4.
    pub fn am4(&self) -> bool {
        if self.mac2() {
            self.op11()
        } else {
            self.na1()
        }
    }
    /// Get the output of the address multiplexer AM3.
    pub fn am3(&self) -> bool {
        if self.mac2() {
            self.op10()
        } else {
            self.am1()
        }
    }
    pub fn am2(&self) -> bool {
        match (self.op01(), self.op00()) {
            (false, false) => true,
            (false, true) => self.carry_flag(),
            (true, false) => self.zero_flag(),
            (true, true) => self.negative_flag(),
        }
    }
    /// Get the output of the address multiplexer AM1.
    pub fn am1(&self) -> bool {
        match (self.mac1(), self.mac0(), self.na0()) {
            (false, false, false) => false,
            (false, false, true) => true,
            (false, true, false) => self.al1(),
            (false, true, true) => self.carry_flag(),
            (true, false, false) => self.carry_out(),
            (true, false, true) => self.zero_out(),
            (true, true, false) => self.negative_out(),
            (true, true, true) => self.il2(),
        }
    }
    /// Get the output of the address logic xor AL1.
    pub fn al1(&self) -> bool {
        self.op10() ^ self.am2()
    }
    /// Get the output of the interrupt logic and IL2.
    pub fn il2(&self) -> bool {
        self.interrupt_enable_flag() && self.interrupt_logic_1()
    }
    /// Get the output of the interrupt logic or IL1.
    pub fn interrupt_logic_1(&self) -> bool {
        self.interrupt_flipflop_1() || self.level_interrupt()
    }
    /// Get the output of the interrupt logic and IL3.
    pub fn interrupt_logic_3(&self) -> bool {
        self.interrupt_flipflop_1() && self.mac1() && self.mac0() && self.na0()
    }
}

impl<'a> From<&'a Machine> for Signals<'a> {
    fn from(machine: &'a Machine) -> Self {
        Signals {
            flags: machine.register.flags(),
            carry_out: machine.alu_output.carry_out(),
            zero_out: machine.alu_output.zero_out(),
            negative_out: machine.alu_output.negative_out(),
            instruction: &machine.instruction_register.get(),
            word: &machine.microprogram_ram.get_word(),
            interrupt_flipflop_1: machine.pending_edge_interrupt.as_ref(),
            level_interrupt: machine.pending_level_interrupt.as_ref(),
        }
    }
}
