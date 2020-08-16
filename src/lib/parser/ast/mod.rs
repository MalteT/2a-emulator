pub type Comment = String;
pub type Label = String;

#[cfg(feature = "formatting")]
mod format;
mod trait_impls;

/// A single byte.
/// Either given by a constant or a label.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Constant {
    Constant(u8),
    Label(Label),
}

/// A general source.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Source {
    Register(Register),
    MemAddress(MemAddress),
    Constant(Constant),
    RegisterDI(RegisterDI),
    RegisterDDI(RegisterDDI),
}

/// A general destination.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Destination {
    Register(Register),
    MemAddress(MemAddress),
    RegisterDI(RegisterDI),
    RegisterDDI(RegisterDDI),
}

/// A dereferenced, post-incremented register.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RegisterDI(pub Register);

/// A double dereferenced, post-incremented register.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RegisterDDI(pub Register);

/// The different stack sizes the Stack may have.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Stacksize {
    /// 16 byte stack.
    _16,
    /// 32 byte stack.
    _32,
    /// 48 byte stack.
    _48,
    /// 64 byte stack.
    _64,
    /// Unlimited stack.
    NotSet,
}

/// Possible register values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Register {
    /// Register 0.
    R0,
    /// Register 1.
    R1,
    /// Register 2.
    R2,
    /// Register 3 and program counter (PC).
    R3,
}

/// Memory address.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum MemAddress {
    /// Dereferencing a constant.
    Constant(Constant),
    /// Dereferencing a register.
    Register(Register),
}

/// Possible instructions for the assembler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    /// Set program origin.
    AsmOrigin(u8),
    /// Leave space for n bytes.
    AsmByte(u8),
    /// Define multiple bytes.
    AsmDefineBytes(Vec<Constant>),
    /// Make label equivalent to constant.
    AsmEquals(Label, Constant),
    /// Define stacksize.
    AsmStacksize(Stacksize),
    /// Clear the register.
    Clr(Register),
    /// Add the second register to the first.
    Add(Register, Register),
    /// Add the second register to the first, with respect to carry.
    Adc(Register, Register),
    /// Subtract the second register from the first.
    Sub(Register, Register),
    /// Multiply the first register by the second.
    Mul(Register, Register),
    /// Divide the first register by the second.
    Div(Register, Register),
    /// Increase the register by 1.
    Inc(Register),
    /// Decrease the source by 1.
    Dec(Source),
    /// Negate a register. (twos complement)
    Neg(Register),
    /// Logic and between two registers.
    And(Register, Register),
    /// Logic or between two registers.
    Or(Register, Register),
    /// Logic xor between two registers.
    Xor(Register, Register),
    /// Complement one register. (ones complement)
    Com(Register),
    /// Set bits from source in destination.
    Bits(Destination, Source),
    /// Clear bits from source in destination.
    Bitc(Destination, Source),
    /// Set flags for register.
    Tst(Register),
    /// Compare source with destination.
    Cmp(Destination, Source),
    /// Bit test source with destination.
    Bitt(Destination, Source),
    /// Logical shift right.
    Lsr(Register),
    /// Arithmetic shift right.
    Asr(Register),
    /// Logical shift left.
    Lsl(Register),
    /// Rotate right through carry.
    Rrc(Register),
    /// Rotate left through carry.
    Rlc(Register),
    /// Move source to destination. (copy)
    Mov(Destination, Source),
    /// Load a constant into a register.
    LdConstant(Register, Constant),
    /// Load a byte from memory into a register.
    LdMemAddress(Register, MemAddress),
    /// Store a register in RAM.
    St(MemAddress, Register),
    /// Push register to stack.
    Push(Register),
    /// Pop register from stack.
    Pop(Register),
    /// Push flag register to stack.
    PushF,
    /// Pop flag register from stack.
    PopF,
    /// Load stack pointer from source.
    Ldsp(Source),
    /// Load flag register from source.
    Ldfr(Source),
    /// Jump.
    Jmp(Label),
    /// Jump if carry set.
    Jcs(Label),
    /// Jump if carry cleared.
    Jcc(Label),
    /// Jump if zero set.
    Jzs(Label),
    /// Jump if zero cleared.
    Jzc(Label),
    /// Jump if negative set.
    Jns(Label),
    /// Jump if negative cleared.
    Jnc(Label),
    /// Jump relative.
    Jr(Label),
    /// Call a subroutine.
    Call(Label),
    /// Return from a subroutine.
    Ret,
    /// Return from interrupt.
    RetI,
    /// Stop the CPU.
    Stop,
    /// No operation.
    Nop,
    /// Enable interrupts.
    Ei,
    /// Disable interrupts.
    Di,
}

/// A single line in the ASM program.
///
/// Either a [`Label`] or an [`Instruction`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Line {
    Empty(Option<Comment>),
    Label(Label, Option<Comment>),
    Instruction(Instruction, Option<Comment>),
}

/// Represenation of a Minirechner2a ASM program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asm {
    pub comment_after_shebang: Option<Comment>,
    pub lines: Vec<Line>,
}
