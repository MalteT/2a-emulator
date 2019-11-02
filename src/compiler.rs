//! Assembler to byte code compilation module.
//! ```text
//!           ;     .ORG 0x00
//!
//!        04 ;     CLR R0
//!           ; LOOP:
//!  F0 1F FF ;     ST (0xFF), R0
//!  F0 1F F0 ;     ST (0xF0), R0
//!        44 ;     INC R0
//!     20 F7 ;     JR LOOP
//! ```

use colored::Colorize;
use parser2a::asm::{
    Asm, Comment, Constant, Destination, Instruction, Label, Line, MemAddress, Register,
    RegisterDDI, RegisterDI, Source,
};

use std::collections::HashMap;
use std::fmt;
use std::ops::Deref;
use std::rc::Rc;

#[derive(Clone)]
/// An either type for [`u8`]/[`Label`].
///
/// This is used for label references.
/// During translation all labels will be translated into
/// this type which is, after all Labels are defined, translated into the correct bytes.
pub enum ByteOrLabel {
    /// An ordinary byte.
    Byte(u8),
    /// A label that will be replaced by the address of the following byte.
    Label(Label),
    /// A label that will be replaced by the address of the following byte
    /// which will the be transformed by the function.
    LabelFn(Label, Rc<dyn Fn(u8) -> u8>),
}

#[derive(Debug, Clone)]
/// This is the final byte code with additional information from which [`Line`]
/// the byte code originates.
pub struct ByteCode {
    /// Lines with translated byte code.
    pub lines: Vec<(Line, Vec<u8>)>,
}

// # TODO: Handle Stacksize
#[derive(Debug, Clone)]
/// Translator for [`Asm`] -> [`ByteCode`]
pub struct Translator {
    next_addr: u8,
    known_labels: HashMap<Label, u8>,
    bytes: Vec<(Line, Vec<ByteOrLabel>)>,
}

impl ByteCode {
    /// Get an iterator over the byte code.
    /// This iterator always starts at address zero.
    pub fn bytes<'a>(&'a self) -> impl Iterator<Item = &u8> + 'a {
        self.lines.iter().map(|(_, c)| c).flatten()
    }
}

impl Translator {
    /// Compile the given [`Asm`] into [`ByteCode`].
    pub fn compile(asm: &Asm) -> ByteCode {
        let mut tr = Translator::new();
        for line in &asm.lines {
            tr.push(line);
        }
        tr.finish()
    }
    /// Create a new translator, starting at address `0` without any Labels and
    /// no bytes in memory.
    fn new() -> Self {
        Translator {
            bytes: vec![],
            known_labels: HashMap::new(),
            next_addr: 0,
        }
    }
    /// Push a [`Line`] into the translator, adding the translated bytes,
    /// changing address and pushing labels
    fn push(&mut self, line: &Line) {
        match line {
            Line::Empty(_) => {
                self.bytes.push((line.clone(), vec![]));
            }
            Line::Label(label, _) => {
                self.known_labels.insert(label.to_string(), self.next_addr);
                self.bytes.push((line.clone(), vec![]));
            }
            Line::Instruction(inst, comment) => self.push_instruction(inst, comment),
        }
    }
    /// Push an instruction into the translator.
    fn push_instruction(&mut self, inst: &Instruction, comment: &Option<Comment>) {
        use ByteOrLabel::*;
        use Instruction::*;
        let bols = match inst.clone() {
            AsmOrigin(addr) => {
                self.next_addr = addr;
                vec![]
            }
            AsmByte(nr) => {
                self.next_addr += nr;
                let mut ret = vec![];
                for _ in 0..nr {
                    ret.push(Byte(0b0000_00_00));
                }
                ret
            }
            AsmDefineBytes(mut cs) => cs.drain(..).map(ByteOrLabel::from).collect(),
            AsmDefineWords(_ws) => unimplemented!(".DW is not yet implemented"),
            // TODO: Research how mcontrol does .EQU translation
            AsmEquals(label, c) => {
                // Push Label!
                self.known_labels.insert(label, self.next_addr);
                vec![c.into()]
            }
            AsmStacksize(_ss) => vec![],
            Clr(reg) => {
                let reg: u8 = reg_to_u8(reg);
                vec![Byte(0b0000_01_00 + reg)]
            }
            Add(rd, rs) => from_base_and_two_regs(0b0110_00_00, rd, rs),
            Adc(rd, rs) => from_base_and_two_regs(0b0111_00_00, rd, rs),
            Sub(rd, rs) => from_base_and_two_regs(0b1000_00_00, rd, rs),
            Mul(rd, rs) => from_base_and_two_regs(0b1011_00_00, rd, rs),
            Div(rd, rs) => from_base_and_two_regs(0b1100_00_00, rd, rs),
            Inc(reg) => from_base_and_reg(0b0100_01_00, reg),
            Dec(src) => match src {
                Source::Register(reg) => from_base_and_reg(0b0101_00_00, reg),
                _ => unimplemented!("DEC [something other than R*] does not work yet"),
            },
            Neg(reg) => from_base_and_reg(0b0011_01_00, reg),
            And(rd, rs) => from_base_and_two_regs(0b1001_00_00, rd, rs),
            Or(rd, rs) => from_base_and_two_regs(0b1010_00_00, rd, rs),
            Xor(rd, rs) => from_base_and_two_regs(0b1101_00_00, rd, rs),
            Com(reg) => from_base_and_reg(0b0011_00_00, reg),
            Bits(dst, src) => from_bases_dst_and_src(0b1111_00_00, 0b0101_00_00, &dst, &src),
            Bitc(dst, src) => from_bases_dst_and_src(0b1111_00_00, 0b0110_00_00, &dst, &src),
            Tst(reg) => from_base_and_reg(0b0110_10_00, reg),
            Cmp(dst, src) => from_bases_dst_and_src(0b1111_00_00, 0b0010_00_00, &dst, &src),
            Bitt(dst, src) => from_bases_dst_and_src(0b1111_00_00, 0b0011_00_00, &dst, &src),
            Lsr(reg) => from_base_and_reg(0b0011_10_00, reg),
            Asr(reg) => from_base_and_reg(0b0011_11_00, reg),
            Lsl(reg) => {
                let reg = reg_to_u8(reg);
                vec![Byte(0b0110_00_00 + (reg << 2) + reg)]
            }
            Rrc(reg) => from_base_and_reg(0b0100_00_00, reg),
            Rlc(reg) => {
                let reg = reg_to_u8(reg);
                vec![Byte(0b0111_00_00 + (reg << 2) + reg)]
            }
            Mov(dst, src) => compile_instruction_mov(dst, src),
            LdConstant(reg, c) => compile_instruction_mov(reg.into(), c.into()),
            LdMemAddress(reg, mem) => compile_instruction_mov(reg.into(), mem.into()),
            St(mem, reg) => compile_instruction_mov(mem.into(), reg.into()),
            Push(reg) => from_base_and_reg(0b0001_00_00, reg),
            Pop(reg) => from_base_and_reg(0b0001_01_00, reg),
            PushF => vec![Byte(0b0001_10_00)],
            PopF => vec![Byte(0b0001_11_00)],
            Ldsp(src) => from_bases_and_src(0b1111_00_00, 0b0100_00_00, &src),
            Ldfr(src) => from_bases_and_src(0b1111_00_00, 0b0100_01_00, &src),
            Jmp(label) => vec![Byte(0b1111_10_11), Label(label), Byte(0b0001_00_11)],
            Jcs(label) => relative_jump(0b001, label, self.next_addr),
            Jcc(label) => relative_jump(0b101, label, self.next_addr),
            Jzs(label) => relative_jump(0b010, label, self.next_addr),
            Jzc(label) => relative_jump(0b110, label, self.next_addr),
            Jns(label) => relative_jump(0b011, label, self.next_addr),
            Jnc(label) => relative_jump(0b111, label, self.next_addr),
            Jr(label) => relative_jump(0b000, label, self.next_addr),
            Call(label) => vec![Byte(0b0010_10_00), Label(label)],
            Ret => vec![Byte(0b0001_01_11)],
            RetI => vec![Byte(0b0010_11_00)],
            Stop => vec![Byte(0b0000_00_01)],
            Nop => vec![Byte(0b0000_00_10)],
            Ei => vec![Byte(0b0000_10_00)],
            Di => vec![Byte(0b0000_11_00)],
        };
        let line = Line::Instruction(inst.clone(), comment.clone());
        self.next_addr += bols.len() as u8;
        self.bytes.push((line, bols));
    }
    /// Finish the translation.
    /// This replaces all references to labels with the address the
    /// Label was defined at.
    fn finish(mut self) -> ByteCode {
        let labels = self.known_labels;
        let lines = self
            .bytes
            .drain(..)
            .map(|(line, mut bols)| {
                let bytes = bols
                    .drain(..)
                    .map(|bol| match bol {
                        ByteOrLabel::Byte(byte) => byte,
                        ByteOrLabel::Label(label) => *labels
                            .get(&label)
                            .expect("infallible. Labels must be defined"),
                        ByteOrLabel::LabelFn(label, f) => {
                            let b = *labels
                                .get(&label)
                                .expect("infallible. Labels must be defined");
                            f.deref()(b)
                        }
                    })
                    .collect();
                (line, bytes)
            })
            .collect();
        ByteCode { lines }
    }
}

/// Create the necessary [`ByteOrLabel`]s for a relative jump with the given condition.
fn relative_jump(cond: u8, label: Label, curr_addr: u8) -> Vec<ByteOrLabel> {
    use ByteOrLabel::*;
    debug_assert!(cond <= 0b0000_0_111);
    let first = Byte(0b0010_0_000 + cond);
    // Calculate relative offset of the target address.
    let second = LabelFn(
        label,
        Rc::from(move |target: u8| {
            let (pre_jump, _) = curr_addr.overflowing_add(2);
            let (diff, _) = target.overflowing_sub(pre_jump);
            diff
        }),
    );
    vec![first, second]
}

/// Compile a `MOV` instruction.
fn compile_instruction_mov(dst: Destination, src: Source) -> Vec<ByteOrLabel> {
    use ByteOrLabel::*;
    // SOURCE
    // Calculate first byte from register and mode
    let source_addr_mode = source_addr_mode(&src) << 2;
    let source_register = source_register(&src);
    let first = 0b1111_00_00 + source_addr_mode + source_register;
    let mut ret = vec![Byte(first)];
    // Add another byte if we need a constant or an address
    let second = match src {
        Source::Constant(c) => Some(c.into()),
        Source::MemAddress(ref mem) => match mem {
            MemAddress::Constant(c) => Some(c.clone().into()),
            MemAddress::Register(_) => None,
        },
        _ => None,
    };
    if let Some(second) = second {
        ret.push(second)
    }
    // DESTINATION
    // Calculate first byte from register and mode
    let destination_addr_mode = destination_addr_mode(&dst) << 2;
    let destination_register = destination_register(&dst);
    let third = 0b0001_00_00 + destination_addr_mode + destination_register;
    ret.push(Byte(third));
    // Add another byte if we need an address
    let fourth = match dst {
        Destination::MemAddress(ref mem) => match mem {
            MemAddress::Constant(c) => Some(c.clone().into()),
            MemAddress::Register(_) => None,
        },
        _ => None,
    };
    if let Some(fourth) = fourth {
        ret.push(fourth)
    }
    ret
}

/// Convert a [`Register`] to [`u8`]
fn reg_to_u8(reg: Register) -> u8 {
    match reg {
        Register::R0 => 0,
        Register::R1 => 1,
        Register::R2 => 2,
        Register::R3 => 3,
    }
}

/// Get the address mode from a [`Source`].
///
/// | Mode  | Meaning |
/// |-------|---------|
/// | `0 0` | R       |
/// | `0 1` | (R)     |
/// | `1 0` | (R+)    |
/// | `1 1` | ((R+))  |
fn source_addr_mode(src: &Source) -> u8 {
    match src {
        Source::Register(_) => 0b00,
        Source::Constant(_) | Source::RegisterDI(_) => 0b10,
        Source::RegisterDDI(_) => 0b11,
        Source::MemAddress(ref mem) => match mem {
            MemAddress::Register(_) => 0b01,
            MemAddress::Constant(_) => 0b11,
        },
    }
}

/// Get the source register from a [`Source`].
fn source_register(src: &Source) -> u8 {
    match src {
        Source::Register(reg)
        | Source::RegisterDI(RegisterDI(reg))
        | Source::RegisterDDI(RegisterDDI(reg)) => reg_to_u8(*reg),
        Source::Constant(_) => 0b11,
        Source::MemAddress(ref mem) => match mem {
            MemAddress::Register(reg) => reg_to_u8(*reg),
            MemAddress::Constant(_) => 0b11,
        },
    }
}

/// Get the address mode from a [`Destination`]. See [`source_addr_mode`] for more.
fn destination_addr_mode(dst: &Destination) -> u8 {
    match dst {
        Destination::Register(_) => 0b00,
        Destination::RegisterDI(_) => 0b10,
        Destination::RegisterDDI(_) => 0b11,
        Destination::MemAddress(ref mem) => match mem {
            MemAddress::Register(_) => 0b01,
            MemAddress::Constant(_) => 0b11,
        },
    }
}

/// Get the destination register from a [`Destination`].
fn destination_register(dst: &Destination) -> u8 {
    match dst {
        Destination::Register(reg)
        | Destination::RegisterDI(RegisterDI(reg))
        | Destination::RegisterDDI(RegisterDDI(reg)) => reg_to_u8(*reg),
        Destination::MemAddress(ref mem) => match mem {
            MemAddress::Register(reg) => reg_to_u8(*reg),
            MemAddress::Constant(_) => 0b11,
        },
    }
}

/// Create a byte vector from two base bytes, a source and a destination.
/// ```text
/// 0b1101_10_11 [0b10110101] 0b0110_11_00 [0b10111001]
///   -B1- MS RS  addr/const    -B2- MD RD  ---addr---
/// ```
fn from_bases_dst_and_src(b1: u8, b2: u8, dst: &Destination, src: &Source) -> Vec<ByteOrLabel> {
    use ByteOrLabel::*;
    // Calculate first byte from register and mode
    let source_addr_mode = source_addr_mode(&src) << 2;
    let source_register = source_register(&src);
    let first = b1 + source_addr_mode + source_register;
    let mut ret = vec![Byte(first)];
    // Add another byte if we need a constant or an address
    let second = match src {
        Source::Constant(c) => Some(c.clone().into()),
        Source::MemAddress(ref mem) => match mem {
            MemAddress::Constant(c) => Some(c.clone().into()),
            MemAddress::Register(_) => None,
        },
        _ => None,
    };
    if let Some(second) = second {
        ret.push(second)
    }
    // DESTINATION
    // Calculate first byte from register and mode
    let destination_addr_mode = destination_addr_mode(&dst) << 2;
    let destination_register = destination_register(&dst);
    let third = b2 + destination_addr_mode + destination_register;
    ret.push(Byte(third));
    // Add another byte if we need an address
    let fourth = match dst {
        Destination::MemAddress(ref mem) => match mem {
            MemAddress::Constant(c) => Some(c.clone().into()),
            MemAddress::Register(_) => None,
        },
        _ => None,
    };
    if let Some(fourth) = fourth {
        ret.push(fourth)
    }
    ret
}

/// Create a Vector of bytes or labels from two bases and a source.
/// ```text
/// 0b1101_10_11 [0b10110101] 0b0110_11_00
///   -B1- MS RS  addr/const    --BASE 2--
/// ```
fn from_bases_and_src(b1: u8, b2: u8, src: &Source) -> Vec<ByteOrLabel> {
    use ByteOrLabel::*;
    // Calculate first byte from register and mode
    let source_addr_mode = source_addr_mode(&src) << 2;
    let source_register = source_register(&src);
    let first = b1 + source_addr_mode + source_register;
    let mut ret = vec![Byte(first)];
    // Add another byte if we need a constant or an address
    let second = match src {
        Source::Constant(c) => Some(c.clone().into()),
        Source::MemAddress(ref mem) => match mem {
            MemAddress::Constant(c) => Some(c.clone().into()),
            MemAddress::Register(_) => None,
        },
        _ => None,
    };
    if let Some(second) = second {
        ret.push(second)
    }
    ret.push(Byte(b2));
    ret
}

/// Create a byte vector from a base and a destination register.
/// ```text
/// 0b0000_01_11
///   -BASE-- RD
/// ```
fn from_base_and_reg(base: u8, reg: Register) -> Vec<ByteOrLabel> {
    let reg = reg_to_u8(reg);
    vec![ByteOrLabel::Byte(base + reg)]
}

/// Create a byte vector from a base, a destination register and a source register.
/// ```text
/// 0b0000_01_11
///   BASE RS RD
/// ```
fn from_base_and_two_regs(base: u8, dst: Register, src: Register) -> Vec<ByteOrLabel> {
    let dst = reg_to_u8(dst);
    let src = reg_to_u8(src) << 2;
    vec![ByteOrLabel::Byte(base + src + dst)]
}

impl From<Constant> for ByteOrLabel {
    fn from(c: Constant) -> Self {
        match c {
            Constant::Constant(c) => ByteOrLabel::Byte(c),
            Constant::Label(label) => ByteOrLabel::Label(label),
        }
    }
}

impl From<MemAddress> for ByteOrLabel {
    fn from(mem: MemAddress) -> Self {
        match mem {
            MemAddress::Constant(c) => c.into(),
            MemAddress::Register(_reg) => unimplemented!("How to make a const from a register"),
        }
    }
}

impl fmt::Display for ByteCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let max_width = self.lines.iter().map(|(_, bs)| bs.len()).max().unwrap_or(0);
        for (line, code) in &self.lines {
            // Skip semicolon for empty lines
            if let Line::Empty(None) = line {
                writeln!(f, "")?;
                continue;
            }
            let line = format!("; {}", line);
            let code: Vec<_> = code.iter().map(|c| format!("{:>02X}", c)).collect();
            let code_str = format!("{:>width$}", code.join(" "), width = max_width * 3);
            writeln!(f, " {} {}", code_str, line.dimmed())?;
        }
        Ok(())
    }
}

impl fmt::Debug for ByteOrLabel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ByteOrLabel::Byte(b) => write!(f, "Byte({:>02X})", b),
            ByteOrLabel::Label(l) => write!(f, "Label({})", l),
            ByteOrLabel::LabelFn(l, _) => write!(f, "LabelFn({}, [hidden])", l),
        }
    }
}
